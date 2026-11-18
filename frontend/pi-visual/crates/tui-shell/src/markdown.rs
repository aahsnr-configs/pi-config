//! Local CommonMark rendering. Raw source is retained by Transcript, never executed.
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
#[derive(Clone)]
pub struct Row {
    pub line: Line<'static>,
    pub offset: usize,
}
/// Shared measurement/painting pipeline. Offsets index the logical displayed text
/// stream (graphemes), so reflow anchors point to the same text, not a wrapped row.
pub fn layout(raw: &str, width: usize, markdown: bool) -> Vec<Row> {
    let width = width.max(1);
    let safe = pi_rpc::reducer::display_text(raw);
    let mut logical: Vec<Vec<(String, Style)>> = vec![Vec::new()];
    let mut style = Style::default();
    let mut stack = Vec::new();
    let mut code = false;
    let mut links = Vec::new();
    let mut lists: Vec<Option<u64>> = Vec::new();
    fn push(lines: &mut [Vec<(String, Style)>], s: impl Into<String>, style: Style) {
        lines.last_mut().unwrap().push((s.into(), style));
    }
    fn newline(lines: &mut Vec<Vec<(String, Style)>>) {
        lines.push(Vec::new());
    }
    if markdown {
        for event in Parser::new_ext(
            &safe,
            Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
        ) {
            match event {
                Event::Start(tag) => {
                    stack.push(style);
                    match tag {
                        Tag::Heading { .. } => {
                            style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
                        }
                        Tag::Link { dest_url, .. } => {
                            links.push(
                                if ["https://", "http://", "mailto:"]
                                    .iter()
                                    .any(|p| dest_url.starts_with(p))
                                {
                                    dest_url.to_string()
                                } else {
                                    "[blocked target]".into()
                                },
                            );
                            style = style.fg(Color::Blue);
                        }
                        Tag::Strong => style = style.add_modifier(Modifier::BOLD),
                        Tag::Emphasis => style = style.add_modifier(Modifier::ITALIC),
                        Tag::Strikethrough => style = style.add_modifier(Modifier::CROSSED_OUT),
                        Tag::CodeBlock(kind) => {
                            code = true;
                            let lang = match kind {
                                CodeBlockKind::Fenced(s) => s.to_string(),
                                _ => String::new(),
                            };
                            push(
                                &mut logical,
                                format!("code {lang}"),
                                Style::default().fg(Color::Cyan),
                            );
                            newline(&mut logical);
                            style = style.fg(Color::Yellow);
                        }
                        Tag::BlockQuote(_) => push(&mut logical, "│ ", style),
                        Tag::List(n) => lists.push(n),
                        Tag::Item => {
                            let prefix = if let Some(Some(n)) = lists.last_mut() {
                                let text = format!("{n}. ");
                                *n += 1;
                                text
                            } else {
                                "• ".into()
                            };
                            push(&mut logical, prefix, style);
                        }
                        Tag::TableCell => push(&mut logical, "│ ", style),
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        TagEnd::Paragraph
                        | TagEnd::Heading(_)
                        | TagEnd::Item
                        | TagEnd::BlockQuote(_)
                        | TagEnd::TableRow
                        | TagEnd::TableHead => newline(&mut logical),
                        TagEnd::CodeBlock => {
                            code = false;
                            newline(&mut logical)
                        }
                        TagEnd::Link => {
                            if let Some(url) = links.pop() {
                                push(&mut logical, format!(" ({url})"), style);
                            }
                        }
                        TagEnd::List(_) => {
                            lists.pop();
                        }
                        _ => {}
                    }
                    style = stack.pop().unwrap_or_default();
                }
                Event::Text(s) => {
                    for (i, part) in s.split('\n').enumerate() {
                        if i > 0 {
                            newline(&mut logical)
                        }
                        push(&mut logical, part, style);
                    }
                }
                Event::Code(s) => push(&mut logical, s.to_string(), style.fg(Color::Yellow)),
                Event::SoftBreak if !code => push(&mut logical, " ", style),
                Event::SoftBreak | Event::HardBreak => newline(&mut logical),
                Event::Rule => {
                    push(&mut logical, "─".repeat(width.min(80)), style);
                    newline(&mut logical)
                }
                Event::TaskListMarker(done) => {
                    push(&mut logical, if done { "[x] " } else { "[ ] " }, style)
                }
                // Escaped inert HTML; never interpreted as a browser or action.
                Event::Html(s) | Event::InlineHtml(s) => push(&mut logical, s.to_string(), style),
                _ => {}
            }
        }
    } else {
        for (i, line) in safe.split('\n').enumerate() {
            if i > 0 {
                newline(&mut logical)
            }
            push(&mut logical, line, style);
        }
    }
    let mut rows = Vec::new();
    let mut offset = 0;
    for segments in logical {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut col = 0;
        let mut start = offset;
        for (text, style) in segments {
            for grapheme in text.graphemes(true) {
                let expanded = if grapheme == "\t" {
                    " ".repeat(4 - col % 4)
                } else {
                    grapheme.to_owned()
                };
                let cells = expanded.width();
                if col > 0 && col + cells > width {
                    rows.push(Row {
                        line: Line::from(std::mem::take(&mut spans)),
                        offset: start,
                    });
                    start = offset;
                    col = 0;
                }
                let shown = if cells > width {
                    "�".to_owned()
                } else {
                    expanded
                };
                let cells = shown.width();
                if let Some(last) = spans.last_mut().filter(|s| s.style == style) {
                    last.content.to_mut().push_str(&shown);
                } else {
                    spans.push(Span::styled(shown, style));
                }
                col += cells;
                offset += 1;
            }
        }
        rows.push(Row {
            line: Line::from(spans),
            offset: start,
        });
        offset += 1;
    }
    if rows.is_empty() {
        rows.push(Row {
            line: Line::default(),
            offset: 0,
        })
    }
    rows
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn widths_styles_controls_and_unclosed_fences() {
        for w in [1, 2, 8, 40] {
            let rows = layout(
                "# Head\n**bold** *italic* `code`\n\n- [x] task\n> quote\n\n|a|b|\n|-|-|\n|1|2|\n```rust\n中e\u{301}🙂\tlong_unbroken_line\u{1b}]52;bad\u{7}",
                w,
                true,
            );
            assert!(rows.iter().all(|r| r.line.width() <= w));
            assert!(
                rows.iter()
                    .flat_map(|r| r.line.spans.iter())
                    .all(|s| !s.content.contains('\u{1b}'))
            );
            assert!(
                rows.iter()
                    .flat_map(|r| r.line.spans.iter())
                    .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
            );
        }
    }
    #[test]
    fn wrap_offsets_preserve_graphemes() {
        let a = layout("abcdef中e\u{301}🙂", 4, false);
        assert_eq!(a[1].offset, 4);
        assert_eq!(a[0].line.width(), 4);
    }
}
