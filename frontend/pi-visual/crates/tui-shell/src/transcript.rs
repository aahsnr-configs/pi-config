//! Virtual cell viewport; raw records outlive bounded rendered-row caches.
use crate::markdown::{self, Row};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::{collections::HashMap, sync::Arc};
const MAX_ENTRIES: usize = 10000;
const MAX_SOURCE: usize = 16 * 1024 * 1024;
const MAX_ENTRY: usize = 256 * 1024;
const CACHE_CELLS: usize = 8 * 1024 * 1024;
const CACHE_ROWS: usize = 65536;
#[derive(Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub raw: Arc<str>,
    pub markdown: bool,
    pub color: Color,
    pub revision: u64,
}
struct Cached {
    key: (u64, usize),
    rows: Arc<Vec<Row>>,
    cells: usize,
    touched: u64,
}
#[derive(Default)]
pub struct Transcript {
    pub entries: Vec<Entry>,
    prefix: Vec<usize>,
    heights: Vec<Option<usize>>,
    pub width: usize,
    pub height: usize,
    pub top: usize,
    pub following: bool,
    cache: HashMap<String, Cached>,
    clock: u64,
    pub formatted: usize,
    pub painted: usize,
    anchor: Option<(String, usize)>,
}
impl Transcript {
    pub fn new() -> Self {
        Self {
            following: true,
            ..Self::default()
        }
    }
    fn anchor(&mut self) -> Option<(String, usize)> {
        if self.following {
            return None;
        }
        let i = self
            .prefix
            .partition_point(|p| *p <= self.top)
            .saturating_sub(1)
            .min(self.entries.len().saturating_sub(1));
        let id = self.entries.get(i)?.id.clone();
        let local = self.top.saturating_sub(*self.prefix.get(i)?);
        let rows = self.rows(i);
        let offset = rows.get(local).map_or(0, |r| r.offset);
        Some((id, offset))
    }
    pub fn replace(&mut self, entries: Vec<Entry>) -> Result<(), String> {
        if entries.len() > MAX_ENTRIES
            || entries
                .iter()
                .any(|e| e.raw.len() > MAX_ENTRY || e.id.len() > 1024 || e.title.len() > 4096)
            || entries.iter().map(|e| e.raw.len()).sum::<usize>() > MAX_SOURCE
        {
            return Err("transcript source budget exceeded".into());
        }
        let mut ids = std::collections::HashSet::new();
        if entries.iter().any(|e| !ids.insert(e.id.clone())) {
            return Err("duplicate transcript identity".into());
        }
        let anchor = self.anchor.clone().or_else(|| self.anchor());
        let old: HashMap<_, _> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                (
                    e.id.clone(),
                    (e.revision, self.heights.get(i).copied().flatten()),
                )
            })
            .collect();
        self.heights = entries
            .iter()
            .map(|e| {
                old.get(&e.id)
                    .filter(|(r, _)| *r == e.revision)
                    .and_then(|(_, h)| *h)
            })
            .collect();
        self.entries = entries;
        self.cache.retain(|id, _| ids.contains(id));
        self.anchor = anchor;
        self.reindex();
        Ok(())
    }
    fn reindex(&mut self) {
        self.prefix.clear();
        self.prefix.push(0);
        for h in &self.heights {
            self.prefix
                .push(self.prefix.last().unwrap() + h.unwrap_or(3));
        }
        self.clamp();
    }
    fn clamp(&mut self) {
        let end = self.total().saturating_sub(self.height);
        if self.following {
            self.top = end
        } else {
            self.top = self.top.min(end)
        }
    }
    pub fn total(&self) -> usize {
        *self.prefix.last().unwrap_or(&0)
    }
    pub fn latest(&mut self) {
        self.following = true;
        self.clamp();
        self.anchor = None;
    }
    pub fn scroll(&mut self, delta: isize) {
        self.top = self
            .top
            .saturating_add_signed(delta)
            .min(self.total().saturating_sub(self.height));
        self.following = self.top >= self.total().saturating_sub(self.height).saturating_sub(1);
        self.anchor = None;
        if !self.following {
            self.anchor = self.anchor();
        }
    }
    pub fn resize(&mut self, width: usize, height: usize) {
        if width != self.width {
            self.anchor = self.anchor.clone().or_else(|| self.anchor());
            self.width = width.max(1);
            self.heights.fill(None);
            self.cache.clear();
        }
        self.height = height;
        self.reindex();
    }
    fn rows(&mut self, i: usize) -> Arc<Vec<Row>> {
        self.clock += 1;
        let e = &self.entries[i];
        if let Some(c) = self
            .cache
            .get_mut(&e.id)
            .filter(|c| c.key == (e.revision, self.width))
        {
            c.touched = self.clock;
            return c.rows.clone();
        }
        self.formatted += 1;
        let mut rows = markdown::layout(&e.title, self.width.max(1), false);
        for row in &mut rows {
            for span in &mut row.line.spans {
                span.style = span.style.fg(e.color);
            }
        }
        use unicode_segmentation::UnicodeSegmentation;
        let header_offset = pi_rpc::reducer::display_text(&e.title)
            .graphemes(true)
            .count()
            + 1;
        let mut body = markdown::layout(&e.raw, self.width.saturating_sub(2).max(1), e.markdown);
        for r in &mut body {
            r.offset += header_offset;
            r.line
                .spans
                .insert(0, Span::styled("▎ ", Style::default().fg(e.color)));
        }
        rows.extend(body);
        rows.push(Row {
            line: Line::default(),
            offset: usize::MAX,
        });
        let cells = rows
            .iter()
            .map(|r| {
                std::mem::size_of::<Row>()
                    + r.line
                        .spans
                        .iter()
                        .map(|s| std::mem::size_of::<Span>() + s.content.len())
                        .sum::<usize>()
            })
            .sum();
        let rows = Arc::new(rows);
        self.cache.insert(
            e.id.clone(),
            Cached {
                key: (e.revision, self.width),
                rows: rows.clone(),
                cells,
                touched: self.clock,
            },
        );
        while self.cache.values().map(|c| c.cells).sum::<usize>() > CACHE_CELLS
            || self.cache.values().map(|c| c.rows.len()).sum::<usize>() > CACHE_ROWS
        {
            if let Some(id) = self
                .cache
                .iter()
                .filter(|(id, _)| *id != &e.id)
                .min_by_key(|(_, v)| v.touched)
                .map(|(id, _)| id.clone())
            {
                self.cache.remove(&id);
            } else {
                self.cache.remove(&e.id);
                break;
            }
        }
        rows
    }
    /// Bounded work slices permit the event loop to service RPC/input between reflows.
    pub fn measure_batch(&mut self, budget: usize) -> bool {
        let indices: Vec<_> = self
            .heights
            .iter()
            .enumerate()
            .filter(|(_, h)| h.is_none())
            .take(budget)
            .map(|(i, _)| i)
            .collect();
        for i in indices {
            self.heights[i] = Some(self.rows(i).len());
        }
        self.reindex();
        if let Some((id, offset)) = &self.anchor
            && let Some(i) = self.entries.iter().position(|e| &e.id == id)
            && self.heights[..=i].iter().all(Option::is_some)
        {
            let offset = *offset;
            let rows = self.rows(i);
            let row = rows
                .partition_point(|r| r.offset <= offset)
                .saturating_sub(1);
            self.top = self.prefix[i] + row;
            self.clamp();
            self.anchor = None;
        }
        self.heights.iter().any(Option::is_none)
    }
    pub fn visible(&mut self) -> Vec<Line<'static>> {
        self.painted = 0;
        let mut out = Vec::new();
        if self.entries.is_empty() {
            return out;
        }
        let start = self
            .prefix
            .partition_point(|p| *p <= self.top)
            .saturating_sub(1);
        for i in start..self.entries.len() {
            if self.prefix[i] >= self.top + self.height {
                break;
            }
            let rows = self.rows(i);
            let skip = self.top.saturating_sub(self.prefix[i]);
            for r in rows
                .iter()
                .skip(skip)
                .take(self.height.saturating_sub(out.len()))
            {
                out.push(r.line.clone());
            }
            self.painted += 1;
            if out.len() >= self.height {
                break;
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entry(i: usize, text: &str) -> Entry {
        Entry {
            id: format!("e{i}"),
            title: "assistant".into(),
            raw: text.into(),
            markdown: true,
            color: Color::Cyan,
            revision: 0,
        }
    }
    fn settle(t: &mut Transcript) {
        while t.measure_batch(64) {}
    }
    #[test]
    fn virtualization_sizes_and_anchors() {
        for n in [0, 1, 100, 1000, 10000] {
            let mut t = Transcript::new();
            t.replace((0..n).map(|i| entry(i, "line one\n\nline two")).collect())
                .unwrap();
            t.resize(60, 18);
            settle(&mut t);
            t.scroll(-100);
            let before = t.top;
            let mut entries = t.entries.clone();
            if let Some(last) = entries.last_mut() {
                last.raw = "stream\n\nmore\n\nrows".into();
                last.revision += 1;
            }
            t.replace(entries).unwrap();
            settle(&mut t);
            if n > 100 {
                assert_eq!(t.top, before)
            }
            let formatted = t.formatted;
            assert!(t.visible().len() <= 18);
            assert!(t.painted <= 18);
            t.visible();
            assert!(t.formatted - formatted <= 18);
            t.latest();
            assert_eq!(t.top, t.total().saturating_sub(18));
        }
    }
    #[test]
    fn reflow_preserves_logical_position_and_partial_clip() {
        let mut t = Transcript::new();
        t.replace(vec![entry(0, &"abcdef中🙂 ".repeat(100))])
            .unwrap();
        t.resize(40, 12);
        settle(&mut t);
        t.scroll(-5);
        let anchor = t.anchor().unwrap();
        t.resize(60, 18);
        settle(&mut t);
        assert_eq!(t.entries[0].id, anchor.0);
        let rows = t.rows(0);
        assert!(rows[t.top].offset <= anchor.1);
        assert!(t.visible().len() <= 18);
    }
}
#[cfg(test)]
mod limits_tests {
    use super::*;
    #[test]
    fn large_code_cache_revisions_resize_and_clipping() {
        let raw = format!("```rust\n{}\n```", "中e\u{301}🙂\t".repeat(5000));
        let mut t = Transcript::new();
        t.replace(vec![Entry {
            id: "large".into(),
            title: "assistant".into(),
            raw: raw.clone().into(),
            markdown: true,
            color: Color::Cyan,
            revision: 0,
        }])
        .unwrap();
        t.resize(40, 12);
        while t.measure_batch(32) {}
        let before = t.formatted;
        t.visible();
        t.visible();
        assert_eq!(t.formatted, before);
        assert_eq!(t.entries[0].raw.as_ref(), raw);
        t.scroll(-100);
        let visible = t.visible();
        assert_eq!(visible.len(), 12);
        assert!(visible.iter().all(|l| l.width() <= 40));
        let mut changed = t.entries.clone();
        changed[0].revision += 1;
        changed[0].raw = "short".into();
        t.replace(changed).unwrap();
        while t.measure_batch(32) {}
        assert!(t.formatted > before);
        assert!(t.total() < 10);
        t.resize(60, 18);
        while t.measure_batch(32) {}
        assert!(t.cache.values().map(|c| c.cells).sum::<usize>() <= CACHE_CELLS);
        assert!(t.cache.values().map(|c| c.rows.len()).sum::<usize>() <= CACHE_ROWS);
    }
    #[test]
    fn rejected_updates_preserve_existing_data() {
        let mut t = Transcript::new();
        let e = Entry {
            id: "x".into(),
            title: "user".into(),
            raw: "old".into(),
            markdown: false,
            color: Color::White,
            revision: 0,
        };
        t.replace(vec![e.clone()]).unwrap();
        assert!(t.replace(vec![e.clone(), e.clone()]).is_err());
        assert_eq!(t.entries.len(), 1);
        let mut huge = e;
        huge.raw = "x".repeat(MAX_ENTRY + 1).into();
        assert!(t.replace(vec![huge]).is_err());
        assert_eq!(t.entries[0].raw.as_ref(), "old");
    }
}
#[cfg(test)]
mod eviction_tests {
    use super::*;
    #[test]
    fn evicted_anchor_is_resolved_before_reflow() {
        let mut t = Transcript::new();
        t.replace(vec![Entry {
            id: "one".into(),
            title: "assistant".into(),
            raw: "abcdef中🙂 ".repeat(300).into(),
            markdown: false,
            color: Color::Cyan,
            revision: 0,
        }])
        .unwrap();
        t.resize(40, 12);
        while t.measure_batch(32) {}
        t.scroll(-30);
        let expected = t.anchor().unwrap();
        t.cache.clear();
        t.resize(60, 18);
        while t.measure_batch(32) {}
        let rows = t.rows(0);
        assert!(rows[t.top].offset <= expected.1);
        assert!(expected.1 - rows[t.top].offset < 60);
    }
}

#[cfg(test)]
mod title_tests {
    use super::*;
    #[test]
    fn long_titles_wrap_without_losing_cells() {
        let mut t = Transcript::new();
        t.replace(vec![Entry {
            id: "long".into(),
            title: "long tool 中🙂 ".repeat(20),
            raw: "body".into(),
            markdown: false,
            color: Color::Cyan,
            revision: 0,
        }])
        .unwrap();
        t.resize(40, 100);
        while t.measure_batch(32) {}
        assert!(t.visible().iter().all(|line| line.width() <= 40));
    }
}
