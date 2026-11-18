//! Ordered, event-derived frontend state. No transport, shell or model calls in draw.
use crate::{
    modal::Modal,
    transcript::{Entry, Transcript},
};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use pi_rpc::{
    RpcEvent,
    extension_ui::{Answer, UiState},
    reducer::{WorkflowStore, display_text},
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
#[derive(Clone)]
enum Slot {
    Message(usize),
    Tool(String),
    Notice(String, Value),
}
pub enum Action {
    None,
    Exit,
    Prompt(String),
    Abort,
    Restart,
    Reply(String, Answer),
}
pub struct App {
    pub transcript: Transcript,
    pub store: WorkflowStore,
    pub modal: Modal,
    pub ui: Arc<Mutex<UiState>>,
    slots: Vec<Slot>,
    notice_bytes: usize,
    pub status: String,
    pub session: String,
    pub offline: bool,
    pub connected: bool,
    pub dirty: bool,
    pub loading: bool,
    show_event_details: bool,
    focus_input: bool,
    selection: Option<(usize, usize)>,
    region: Rect,
    pub rows: Vec<Line<'static>>,
    notice: String,
    cwd: String,
}
fn message_text(m: &Value) -> String {
    if let Some(s) = m["content"].as_str() {
        return s.into();
    }
    m["content"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|b| b["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
fn excerpt(s: &str, n: usize) -> String {
    let mut chars = s.chars();
    let mut out: String = chars.by_ref().take(n).collect();
    if chars.next().is_some() {
        out.push_str("\n[excerpt; original retained in workflow record]");
    }
    out
}
fn event_summary(v: &Value) -> (String, String) {
    let mut fields = Vec::new();
    let title = match v["type"].as_str() {
        Some("auto_retry_start") => "Retry started",
        Some("auto_retry_end") => match v["success"].as_bool() {
            Some(true) => "Retry finished · succeeded",
            Some(false) => "Retry finished · failed",
            None => "Retry finished · result unknown",
        },
        Some("agent_settled") if v["aborted"] == true => {
            return (
                "run aborted (backend)".into(),
                "Pi reported that the run was aborted.".into(),
            );
        }
        _ => {
            return (
                v["type"].as_str().unwrap_or("Observed event").into(),
                excerpt(&v.to_string(), 4096),
            );
        }
    };
    if let Some(attempt) = v["attempt"].as_u64() {
        let maximum = v["maxAttempts"]
            .as_u64()
            .map(|n| format!(" of {n}"))
            .unwrap_or_default();
        fields.push(format!("Attempt: {attempt}{maximum}"));
    }
    if let Some(delay) = v["delayMs"].as_u64() {
        fields.push(format!("Delay: {delay} ms"));
    }
    for key in ["errorMessage", "finalError"] {
        if let Some(error) = v[key].as_str() {
            fields.push(format!("Error: {}", excerpt(error, 4096)));
        }
    }
    (title.into(), fields.join("\n"))
}
impl App {
    pub fn new(offline: bool, ui: Arc<Mutex<UiState>>) -> Self {
        Self {
            transcript: Transcript::new(),
            store: WorkflowStore::with_entry_capacity(10000).expect("bounded frontend capacity"),
            modal: Modal::default(),
            ui,
            slots: Vec::new(),
            notice_bytes: 0,
            status: if offline {
                "DEMO / OFFLINE · no RPC or provider requests".into()
            } else {
                "RPC connected".into()
            },
            session: "local".into(),
            offline,
            connected: !offline,
            dirty: true,
            loading: false,
            show_event_details: false,
            focus_input: true,
            selection: None,
            region: Rect::default(),
            rows: Vec::new(),
            notice: String::new(),
            cwd: std::env::current_dir()
                .map(|p| display_text(&p.display().to_string()))
                .unwrap_or_default(),
        }
    }
    pub fn ingest(&mut self, e: &RpcEvent) -> Result<(), String> {
        if e.kind == "extension_ui_request" && self.offline {
            self.ui
                .lock()
                .unwrap()
                .ingest(&e.payload, std::time::Instant::now())
                .map_err(|e| e.to_string())?;
        }
        self.store.apply(e).map_err(|e| e.to_string())?;
        let snapshot = self.store.snapshot();
        match e.kind.as_str() {
            "message_start" | "message_end" => {
                if !snapshot.messages.is_empty() {
                    let i = snapshot.messages.len() - 1;
                    let m = &snapshot.messages[i];
                    if m["role"] != "toolResult"
                        && !self
                            .slots
                            .iter()
                            .any(|s| matches!(s,Slot::Message(n) if *n==i))
                    {
                        self.slots.push(Slot::Message(i));
                    }
                }
            }
            "tool_execution_start" | "tool_execution_update" | "tool_execution_end" => {
                if let Some(id) = e.payload["toolCallId"].as_str()
                    && !self
                        .slots
                        .iter()
                        .any(|s| matches!(s,Slot::Tool(t) if t==id))
                {
                    self.slots.push(Slot::Tool(id.into()));
                }
            }
            "auto_retry_start" | "auto_retry_end" | "compaction_start" | "compaction_end" => {
                self.push_notice(&e.payload)?
            }
            "agent_settled" if e.payload["aborted"] == true => self.push_notice(&e.payload)?,
            "message_update"
            | "agent_start"
            | "agent_end"
            | "agent_settled"
            | "turn_start"
            | "turn_end"
            | "extension_ui_request" => {}
            _ => self.push_notice(&e.payload)?,
        }

        self.dirty = true;
        Ok(())
    }
    fn push_notice(&mut self, payload: &Value) -> Result<(), String> {
        let bytes = serde_json::to_vec(payload)
            .map_err(|e| e.to_string())?
            .len();
        let total = self
            .notice_bytes
            .checked_add(bytes)
            .filter(|n| *n <= 8 * 1024 * 1024)
            .ok_or("workflow notice budget exceeded")?;
        if self.slots.len() >= 10000 {
            return Err("workflow slot budget exceeded".into());
        }
        self.slots.push(Slot::Notice(
            format!("event-{}", self.slots.len()),
            payload.clone(),
        ));
        self.notice_bytes = total;
        Ok(())
    }
    /// Snapshot only when ingesting/restoring, never on repaint or visual interaction.
    pub fn reconcile(&mut self) -> Result<(), String> {
        let s = self.store.snapshot();
        let previous: std::collections::HashMap<_, _> = self
            .transcript
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e))
            .collect();
        let durable_ids: Vec<_> = s
            .branch
            .iter()
            .filter(|id| s.entries[*id].get("message").is_some())
            .collect();
        let mut entries = Vec::new();
        for slot in &self.slots {
            let (id, title, raw, markdown, color) = match slot {
                Slot::Message(i) => {
                    let Some(m) = s.messages.get(*i) else {
                        continue;
                    };
                    let role = m["role"].as_str().unwrap_or("message");
                    let durable = durable_ids.get(*i).copied();
                    (
                        durable.map_or_else(|| format!("message-{i}"), Clone::clone),
                        if role == "toolResult" {
                            format!(
                                "{} · {}",
                                m["toolName"].as_str().unwrap_or("tool result"),
                                match m["isError"].as_bool() {
                                    Some(true) => "failed",
                                    Some(false) => "completed",
                                    None => "status unknown",
                                }
                            )
                        } else {
                            role.to_string()
                        },
                        message_text(m),
                        role == "assistant",
                        if role == "user" {
                            Color::Magenta
                        } else {
                            Color::Cyan
                        },
                    )
                }
                Slot::Tool(id) => {
                    let Some(t) = s.tools.get(id) else { continue };
                    let name = t["toolName"].as_str().unwrap_or("unknown tool");
                    let (status, color) = match t["type"].as_str() {
                        Some("tool_execution_end") => match t["isError"].as_bool() {
                            Some(true) => ("failed", Color::Red),
                            Some(false) => ("completed", Color::Green),
                            None => ("finished (status unknown)", Color::Yellow),
                        },
                        _ => ("in progress (last observed)", Color::Blue),
                    };
                    let mut raw = String::new();
                    for key in ["path", "command"] {
                        if let Some(value) = t["args"][key].as_str() {
                            raw.push_str(&format!("{key}: {}\n", excerpt(value, 4096)));
                        }
                    }
                    let result = t.get("result").or_else(|| t.get("partialResult"));
                    if let Some(r) = result {
                        raw.push_str(&excerpt(&message_text(r), 32768));
                        if let Some(err) = r.get("error") {
                            raw.push_str(&excerpt(&err.to_string(), 4096));
                        }
                    }
                    if let Some(details) = result.and_then(|r| r.get("details")) {
                        raw.push_str(&format!(
                            "\nDetails: {}",
                            excerpt(&details.to_string(), 4096)
                        ));
                    }
                    if t["durationMs"]
                        .as_f64()
                        .is_some_and(|n| n.is_finite() && n >= 0.0)
                    {
                        let duration = &t["durationMs"];
                        raw.push_str(&format!("\nDuration: {duration} ms (backend)"));
                    }
                    (
                        format!("tool-{id}"),
                        format!("{name} · {status}"),
                        raw,
                        false,
                        color,
                    )
                }
                Slot::Notice(id, v) => {
                    let (title, mut body) = event_summary(v);
                    if self.show_event_details {
                        body.push_str(&format!("\nRaw event:\n{}", excerpt(&v.to_string(), 4096)));
                    }
                    (id.clone(), title, body, false, Color::Yellow)
                }
            };
            let id = format!("{}:{id}", self.session);
            let prior = previous.get(id.as_str());
            let revision = prior.map_or(0, |e| {
                e.revision + u64::from(e.raw.as_ref() != raw || e.title != title)
            });
            entries.push(Entry {
                id,
                title,
                raw: raw.into(),
                markdown,
                color,
                revision,
            });
        }
        // Backend text UI is view state, not messages or executable components.
        // Preserve each documented key; the wire provides no extension origin.
        let ui = self.ui.lock().unwrap().clone();
        let keyed = |kind: &str, key: &str| {
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            key.hash(&mut hash);
            format!("{kind}/{:016x}", hash.finish())
        };
        let mut text_ui = Vec::new();
        for (key, text) in &ui.status {
            text_ui.push((
                keyed("status", key),
                format!("Extension status · {}", excerpt(key, 512)),
                text.clone(),
            ));
        }
        for (key, (lines, _placement)) in &ui.widgets {
            text_ui.push((
                keyed("widget", key),
                format!("Extension text widget · {}", excerpt(key, 512)),
                excerpt(&lines.join("\n"), 32768),
            ));
        }
        if let Some(title) = ui.title {
            text_ui.push(("title".into(), "Extension title".into(), title));
        }
        for (index, (text, kind)) in ui.notifications.iter().enumerate() {
            text_ui.push((
                format!("notification/{index}"),
                format!("Extension notification · {kind}"),
                text.clone(),
            ));
        }
        for (key, title, raw) in text_ui {
            let id = format!("ui:{}:{key}", self.session);
            let revision = previous.get(id.as_str()).map_or(0, |e| {
                e.revision + u64::from(e.raw.as_ref() != raw || e.title != title)
            });
            entries.push(Entry {
                id,
                title,
                raw: raw.into(),
                markdown: false,
                color: Color::Yellow,
                revision,
            });
        }
        self.transcript.replace(entries)?;
        self.selection = None;
        Ok(())
    }
    pub fn restore(&mut self, session: &str, data: &Value) -> Result<(), String> {
        self.store
            .restore(session, data, false)
            .map_err(|e| e.to_string())?;
        self.session = session.into();
        self.notice_bytes = 0;
        self.slots = (0..self.store.snapshot().messages.len())
            .map(Slot::Message)
            .collect();
        self.transcript = Transcript::new();
        self.reconcile()?;
        self.dirty = true;
        Ok(())
    }
    pub fn set_directory(&mut self, path: &std::path::Path) {
        self.cwd = display_text(&path.display().to_string());
    }
    pub fn disconnected(&mut self) {
        self.connected = false;
        self.ui.lock().unwrap().invalidate();
        self.modal.sync(&self.ui);
        self.status =
            "RPC disconnected · input disabled; reconnect requires explicit session restoration"
                .into();
        self.dirty = true;
    }
    pub fn input(&mut self, event: Event) -> Action {
        self.modal.sync(&self.ui);
        self.dirty = true;
        if self.modal.active.is_some() {
            if let Event::Paste(text) = &event {
                self.modal.paste(text);
                return Action::None;
            }
            if let Event::Key(k) = event
                && let Some((id, answer)) = self.modal.key(k)
            {
                return Action::Reply(id, answer);
            }
            return Action::None;
        }
        match event {
            Event::Paste(text) if self.focus_input => {
                for c in display_text(&text).chars() {
                    if self.modal.draft.len() + c.len_utf8() > 8192 {
                        break;
                    }
                    self.modal.draft.push(c);
                }
            }
            Event::Resize(w, h) => {
                self.resize(w, h);
            }
            Event::Key(k) if k.kind == KeyEventKind::Press => {
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    return Action::Exit;
                }
                if k.code == KeyCode::Char('g') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    return Action::Abort;
                }
                if k.code == KeyCode::Char('r') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    if let Some(text) = self.ui.lock().unwrap().editor_text.take() {
                        self.modal.draft = text;
                    }
                    return Action::None;
                }
                if k.code == KeyCode::F(5) && !self.offline {
                    return Action::Restart;
                }
                if k.code == KeyCode::F(3) {
                    self.show_event_details = !self.show_event_details;
                    if let Err(error) = self.reconcile() {
                        self.status = error;
                    }
                    return Action::None;
                }
                match k.code {
                    KeyCode::Char('u')
                        if self.focus_input && k.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.modal.draft.clear()
                    }
                    KeyCode::PageUp => self.transcript.scroll(-(self.transcript.height as isize)),
                    KeyCode::PageDown => self.transcript.scroll(self.transcript.height as isize),
                    KeyCode::End if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.transcript.latest()
                    }
                    KeyCode::F(6) => self.focus_input = !self.focus_input,
                    KeyCode::Up if !self.focus_input => self.transcript.scroll(-1),
                    KeyCode::Down if !self.focus_input => self.transcript.scroll(1),
                    KeyCode::Char(c)
                        if self.focus_input
                            && !k
                                .modifiers
                                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                            && self.modal.draft.len() < 8192 =>
                    {
                        self.modal.draft.push(c)
                    }
                    KeyCode::Backspace if self.focus_input => {
                        use unicode_segmentation::UnicodeSegmentation;
                        if let Some((i, _)) = self.modal.draft.grapheme_indices(true).next_back() {
                            self.modal.draft.truncate(i);
                        }
                    }
                    KeyCode::Enter if self.focus_input => {
                        if self.offline {
                            self.status = "OFFLINE: draft retained, nothing sent".into()
                        } else if self.modal.draft == "/restart" {
                            return Action::Restart;
                        } else if self.connected && !self.modal.draft.trim().is_empty() {
                            return Action::Prompt(self.modal.draft.clone());
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp => self.transcript.scroll(-3),
                MouseEventKind::ScrollDown => self.transcript.scroll(3),
                MouseEventKind::Down(MouseButton::Left) => {
                    if self.region.contains((m.column, m.row).into()) {
                        self.focus_input = false;
                        let row = self.transcript.top + (m.row - self.region.y) as usize;
                        self.selection = Some((row, row));
                    } else {
                        self.focus_input = true;
                        self.selection = None;
                    }
                }
                MouseEventKind::Drag(MouseButton::Left)
                    if self.region.contains((m.column, m.row).into()) =>
                {
                    if let Some((_, end)) = &mut self.selection {
                        *end = self.transcript.top + (m.row - self.region.y) as usize;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Action::None
    }
    pub fn resize(&mut self, w: u16, h: u16) {
        self.region = Rect::new(0, 1, w, h.saturating_sub(5));
        self.transcript
            .resize(w as usize, self.region.height as usize);
        self.dirty = true;
    }
    pub fn prepare(&mut self) -> bool {
        self.modal.sync(&self.ui);
        self.notice = {
            let state = self.ui.lock().unwrap();
            if state.editor_text.is_some() {
                "Backend editor text staged · Ctrl+R applies; current draft preserved".into()
            } else {
                state
                    .notifications
                    .back()
                    .map(|(s, _)| s.clone())
                    .or_else(|| state.status.iter().next().map(|(k, v)| format!("{k}: {v}")))
                    .unwrap_or_default()
            }
        };
        let more = self.transcript.measure_batch(32);
        self.rows = self.transcript.visible();
        self.loading = more;
        more
    }
    pub fn draw(&self, f: &mut Frame) {
        let a = f.area();
        f.render_widget(
            Paragraph::new(display_text(&self.status)),
            Rect::new(0, 0, a.width, a.height.min(1)),
        );
        for (i, line) in self.rows.iter().enumerate() {
            let row = self.transcript.top + i;
            let selected = self
                .selection
                .is_some_and(|(s, e)| row >= s.min(e) && row <= s.max(e));
            f.render_widget(
                Paragraph::new(line.clone()).style(if selected {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default().bg(Color::DarkGray)
                }),
                Rect::new(
                    self.region.x,
                    self.region.y + i as u16,
                    self.region.width,
                    1,
                ),
            );
        }
        if a.height >= 4 {
            let y = a.height - 4;
            f.render_widget(
                Paragraph::new(display_text(&self.notice)),
                Rect::new(0, y, a.width, 1),
            );
            let draft = format!(
                "{} {}▏",
                if self.focus_input { ">" } else { "·" },
                display_text(&self.modal.draft)
            );
            let draft_rows = crate::markdown::layout(&draft, a.width as usize, false);
            let draft_rows: Vec<_> = draft_rows
                .iter()
                .skip(draft_rows.len().saturating_sub(2))
                .map(|r| r.line.clone())
                .collect();
            f.render_widget(
                Paragraph::new(draft_rows).style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Rect::new(0, y + 1, a.width, 2),
            );
        }
        if a.height > 0 {
            let right = "Cost — · Context —";
            let cwd = &self.cwd;
            use unicode_segmentation::UnicodeSegmentation;
            use unicode_width::UnicodeWidthStr;
            let available = (a.width as usize).saturating_sub(right.width() + 1);
            let mut left = String::new();
            let mut cells = 0;
            for g in cwd.graphemes(true).rev() {
                if cells + g.width() > available {
                    break;
                }
                left.insert_str(0, g);
                cells += g.width();
            }
            let footer = format!(
                "{}{}{right}",
                left,
                " ".repeat((a.width as usize).saturating_sub(cells + right.width()))
            );
            f.render_widget(
                Paragraph::new(footer),
                Rect::new(0, a.height - 1, a.width, 1),
            );
        }
        self.modal.draw_overlay(f);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, MouseEvent};
    use serde_json::json;
    fn event(v: Value) -> RpcEvent {
        RpcEvent {
            kind: v["type"].as_str().unwrap().into(),
            payload: v,
        }
    }
    fn app() -> App {
        App::new(true, Arc::new(Mutex::new(UiState::default())))
    }
    #[test]
    fn dimensions_truthful_cards_raw_and_modal_priority() {
        let mut a = app();
        for line in include_str!("../../../tests/fixtures/p04-transcript.jsonl").lines() {
            a.ingest(&event(serde_json::from_str(line).unwrap()))
                .unwrap();
        }
        a.reconcile().unwrap();
        assert!(
            a.transcript
                .entries
                .iter()
                .any(|e| e.title == "bash · failed")
        );
        assert!(
            !a.transcript
                .entries
                .iter()
                .any(|e| e.title.contains("passed"))
        );
        assert!(
            a.transcript.entries[1]
                .raw
                .contains("**synthetic fixture data**")
        );
        for (w, h) in [
            (40, 12),
            (60, 18),
            (80, 24),
            (100, 30),
            (120, 40),
            (160, 50),
        ] {
            a.resize(w, h);
            while a.prepare() {}
            let mut terminal =
                ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
            terminal.draw(|f| a.draw(f)).unwrap();
            assert!(a.transcript.painted <= a.transcript.height);
            for cell in terminal.backend().buffer().content() {
                assert!(!cell.symbol().contains('\u{1b}'));
            }
        }
        a.modal.draft = "do not lose draft".into();
        a.ingest(&event(json!({"type":"extension_ui_request","id":"p","method":"confirm","title":"Actual request","message":"Actual message"}))).unwrap();
        a.prepare();
        let top = a.transcript.top;
        assert!(matches!(
            a.input(Event::Key(KeyEvent::new(
                KeyCode::PageUp,
                KeyModifiers::NONE
            ))),
            Action::None
        ));
        assert_eq!(a.transcript.top, top);
        assert!(matches!(
            a.input(Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 1,
                row: 2,
                modifiers: KeyModifiers::NONE
            })),
            Action::None
        ));
        assert_eq!(a.transcript.top, top);
        assert_eq!(a.modal.draft, "do not lose draft");
        assert!(matches!(
            a.input(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
            Action::Reply(_, Answer::Cancel)
        ));
    }
    #[test]
    fn history_switch_uses_durable_ids_and_disconnect_preserves_draft() {
        let mut a = app();
        let data = json!({"entries":[{"type":"message","id":"one","parentId":null,"message":{"role":"user","content":"saved"}}],"leafId":"one"});
        a.restore("s1", &data).unwrap();
        assert_eq!(a.transcript.entries[0].id, "s1:one");
        a.modal.draft = "draft".into();
        a.restore("s2", &data).unwrap();
        assert_eq!(a.transcript.entries[0].id, "s2:one");
        assert_eq!(a.modal.draft, "draft");
        a.offline = false;
        a.connected = true;
        a.disconnected();
        assert!(a.status.starts_with("RPC disconnected"));
        assert!(matches!(
            a.input(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE
            ))),
            Action::None
        ));
    }
    #[test]
    fn streaming_does_not_move_reader_and_parallel_order_is_observed() {
        let mut a = app();
        for i in 0..100 {
            a.ingest(&event(json!({"type":"message_start","message":{"role":"user","content":format!("message {i}")}}))).unwrap();
            a.ingest(&event(json!({"type":"message_end","message":{"role":"user","content":format!("message {i}")}}))).unwrap();
        }
        for id in ["z-first", "a-second"] {
            a.ingest(&event(
                json!({"type":"tool_execution_start","toolCallId":id,"toolName":"read","args":{}}),
            ))
            .unwrap();
        }
        a.reconcile().unwrap();
        a.resize(60, 18);
        while a.prepare() {}
        a.transcript.scroll(-60);
        let top = a.transcript.top;
        a.ingest(&event(
            json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
        ))
        .unwrap();
        for _ in 0..20 {
            a.ingest(&event(json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":"piece "}}))).unwrap();
        }
        a.reconcile().unwrap();
        while a.prepare() {}
        assert_eq!(a.transcript.top, top);
        let tail = &a.transcript.entries[100..];
        assert!(tail[0].id.ends_with("z-first"));
        assert!(tail[1].id.ends_with("a-second"));
    }
    #[test]
    fn offline_input_never_creates_prompt_action() {
        let mut a = app();
        a.modal.draft = "hello".into();
        assert!(matches!(
            a.input(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE
            ))),
            Action::None
        ));
        assert_eq!(a.modal.draft, "hello");
    }
    #[test]
    fn extension_text_and_staged_editor_remain_accessible() {
        let mut a = app();
        a.modal.draft = "preserved draft".into();
        for payload in [
            json!({"type":"extension_ui_request","id":"status","method":"setStatus","statusKey":"guard","statusText":"backend warning"}),
            json!({"type":"extension_ui_request","id":"widget","method":"setWidget","widgetKey":"details","widgetLines":["actual text widget"]}),
            json!({"type":"extension_ui_request","id":"title","method":"setTitle","title":"backend title"}),
            json!({"type":"extension_ui_request","id":"editor","method":"set_editor_text","text":"staged replacement"}),
        ] {
            a.ingest(&event(payload)).unwrap();
        }
        a.reconcile().unwrap();
        a.resize(60, 18);
        while a.prepare() {}
        for text in ["backend warning", "actual text widget", "backend title"] {
            assert!(a.transcript.entries.iter().any(|e| e.raw.contains(text)));
        }
        assert_eq!(a.modal.draft, "preserved draft");
        assert!(matches!(
            a.input(Event::Key(KeyEvent::new(
                KeyCode::Char('r'),
                KeyModifiers::CONTROL
            ))),
            Action::None
        ));
        assert_eq!(a.modal.draft, "staged replacement");
    }
    #[test]
    fn key_layout_golden() {
        let mut a = app();
        a.restore("fixture",&json!({"entries":[{"type":"message","id":"m","parentId":null,"message":{"role":"assistant","content":"# Heading\n\nhello **world**"}}],"leafId":"m"})).unwrap();
        a.resize(40, 12);
        while a.prepare() {}
        a.transcript.latest();
        a.rows = a.transcript.visible();
        let actual = a
            .rows
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            actual,
            include_str!("../../../tests/fixtures/p04-basic.golden")
                .strip_suffix('\n')
                .unwrap()
        );
    }
}
#[cfg(test)]
mod burst_tests {
    use super::*;
    #[tokio::test]
    async fn rpc_burst_is_reduced_while_reflow_is_pending() {
        let mut command = tokio::process::Command::new("python3");
        command.env_clear().args(["-u","-c","import json,sys\nfor i in range(100):\n m=dict(role='user',content='burst '+str(i))\n for t in ['message_start','message_end']: print(json.dumps(dict(type=t,message=m)))\nsys.stdin.read()"]);
        let mut client = pi_rpc::client::Client::spawn(
            &mut command,
            pi_rpc::client::Limits {
                event_capacity: 256,
                ..Default::default()
            },
        )
        .unwrap();
        let mut app = App::new(false, client.handle().ui());
        app.resize(40, 12);
        let mut received = 0;
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while received < 200 {
                let event = client.events.recv().await.unwrap();
                app.ingest(&event).unwrap();
                received += 1;
                if received % 32 == 0 {
                    app.reconcile().unwrap();
                    app.transcript.resize(60, 18);
                    app.prepare();
                    tokio::task::yield_now().await;
                }
            }
        })
        .await
        .unwrap();
        app.reconcile().unwrap();
        while app.prepare() {}
        assert_eq!(app.transcript.entries.len(), 100);
        assert!(!client.health.borrow().desynchronized);
        assert!(!client.handle().is_closed());
        client.shutdown().await.unwrap();
    }
}
#[cfg(test)]
mod paste_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn paste_never_approves_and_draft_is_bounded() {
        let mut app = App::new(true, Arc::new(Mutex::new(UiState::default())));
        assert!(matches!(
            app.input(Event::Paste("text\ny".into())),
            Action::None
        ));
        assert_eq!(app.modal.draft, "text\ny");
        app.ingest(&RpcEvent{kind:"extension_ui_request".into(),payload:json!({"type":"extension_ui_request","id":"paste","method":"confirm","title":"t","message":"m"})}).unwrap();
        app.prepare();
        assert!(matches!(
            app.input(Event::Paste("y\n".into())),
            Action::None
        ));
        assert_eq!(
            app.ui.lock().unwrap().dialogs["paste"].phase,
            pi_rpc::extension_ui::Phase::Pending
        );
        assert_eq!(app.modal.draft, "text\ny");
    }
}
#[cfg(test)]
mod large_history_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn ten_thousand_durable_entries_render_as_bounded_view() {
        let entries:Vec<_>=(0..10000).map(|i|json!({"type":"message","id":format!("e{i}"),"parentId":if i==0{None}else{Some(format!("e{}",i-1))},"message":{"role":"assistant","content":"**durable fixture** 中🙂"}})).collect();
        let mut app = App::new(true, Arc::new(Mutex::new(UiState::default())));
        app.restore("saved", &json!({"entries":entries,"leafId":"e9999"}))
            .unwrap();
        app.resize(80, 24);
        while app.prepare() {}
        assert_eq!(app.transcript.entries.len(), 10000);
        assert_eq!(app.transcript.entries[9999].id, "saved:e9999");
        assert!(app.rows.len() <= 19);
        assert!(app.transcript.painted <= 19);
    }
}

#[cfg(test)]
mod acceptance_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn chrome_regions_and_long_draft_remain_visible() {
        for (w, h) in [
            (40, 12),
            (60, 18),
            (80, 24),
            (100, 30),
            (120, 40),
            (160, 50),
        ] {
            let mut a = App::new(true, Arc::new(Mutex::new(UiState::default())));
            a.status = "HEADER".into();
            a.modal.draft = format!("{}END", "中🙂 ".repeat(80));
            a.resize(w, h);
            while a.prepare() {}
            let mut terminal =
                ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
            terminal.draw(|f| a.draw(f)).unwrap();
            let buffer = terminal.backend().buffer();
            let row = |y: u16| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>();
            assert!(row(0).starts_with("HEADER"));
            assert!(row(h - 1).contains("Cost — · Context —"));
            assert!(format!("{}{}", row(h - 3), row(h - 2)).contains("END"));
            assert_eq!(a.region, Rect::new(0, 1, w, h - 5));
        }
    }
    #[test]
    fn known_retry_cards_are_structured_and_originals_retained() {
        let mut a = App::new(true, Arc::new(Mutex::new(UiState::default())));
        let payload = json!({"type":"auto_retry_start","attempt":2,"maxAttempts":3,"delayMs":1200,"errorMessage":"busy","futureDetail":"retained"});
        a.ingest(&RpcEvent {
            kind: "auto_retry_start".into(),
            payload: payload.clone(),
        })
        .unwrap();
        a.reconcile().unwrap();
        let e = &a.transcript.entries[0];
        assert_eq!(e.title, "Retry started");
        assert!(e.raw.contains("Attempt: 2 of 3"));
        assert!(e.raw.contains("Delay: 1200 ms"));
        assert!(!e.raw.contains("{\""));
        assert!(matches!(&a.slots[0],Slot::Notice(_,v) if v==&payload));
        a.ingest(&RpcEvent {
            kind: "future_event".into(),
            payload: json!({"type":"future_event","unknown":"kept"}),
        })
        .unwrap();
        a.reconcile().unwrap();
        assert_eq!(a.transcript.entries[1].title, "future_event");
        assert!(a.transcript.entries[1].raw.contains("kept"));
        let answer = a.input(Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::F(3),
            KeyModifiers::NONE,
        )));
        assert!(matches!(answer, Action::None));
        assert!(a.transcript.entries[0].raw.contains("futureDetail"));
        let count = a.slots.len();
        assert!(
            a.ingest(&RpcEvent {
                kind: "huge_unknown".into(),
                payload: json!({"type":"huge_unknown","raw":"x".repeat(8*1024*1024)})
            })
            .is_err()
        );
        assert_eq!(a.slots.len(), count);
    }
}

#[cfg(test)]
mod ansi_status_tests {
    use super::*;
    use serde_json::json;
    fn event(payload: Value) -> RpcEvent {
        RpcEvent {
            kind: payload["type"].as_str().unwrap().into(),
            payload,
        }
    }
    fn plain(app: &App) -> String {
        app.rows
            .iter()
            .map(|r| {
                r.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    #[test]
    fn ansi_status_updates_replace_by_key_and_raw_values_survive() {
        let mut app = App::new(true, Arc::new(Mutex::new(UiState::default())));
        app.resize(100, 30);
        for n in 0..200 {
            for key in ["extmgr", "pi-lens-lsp"] {
                app.ingest(&event(json!({"type":"extension_ui_request","id":format!("s-{key}-{n}"),"method":"setStatus","statusKey":key,"statusText":format!("\x1b[38;2;126;136;142mDEMO {key} {n}\x1b[39m")}))).unwrap();
            }
            app.reconcile().unwrap();
            while app.prepare() {}
            assert_eq!(app.transcript.entries.len(), 2);
        }
        let shown = plain(&app);
        assert!(shown.contains("DEMO extmgr 199"));
        assert!(shown.contains("DEMO pi-lens-lsp 199"));
        assert!(!shown.contains("[38;"));
        assert!(!shown.contains("[39m"));
        assert!(!shown.contains('\x1b'));
        assert!(app.ui.lock().unwrap().status["extmgr"].contains('\x1b'));
        assert!(app.transcript.entries[0].raw.contains('\x1b'));
        assert!(app.ui.lock().unwrap().dialogs.is_empty());
        assert!(app.store.snapshot().messages.is_empty());
        app.ingest(&event(json!({"type":"extension_ui_request","id":"remove","method":"setStatus","statusKey":"extmgr"}))).unwrap();
        app.reconcile().unwrap();
        assert_eq!(app.transcript.entries.len(), 1);
        assert_eq!(app.ui.lock().unwrap().status.len(), 1);
    }
    #[test]
    fn restored_quit_is_history_and_leaves_draft_and_connection_intact() {
        let mut app = App::new(false, Arc::new(Mutex::new(UiState::default())));
        app.modal.draft = "unsent draft".into();
        app.restore("saved",&json!({"entries":[{"type":"message","id":"quit","parentId":null,"message":{"role":"user","content":"/quit"}}],"leafId":"quit"})).unwrap();
        app.resize(60, 18);
        while app.prepare() {}
        assert!(plain(&app).contains("/quit"));
        assert_eq!(app.modal.draft, "unsent draft");
        assert!(app.connected);
        assert_eq!(app.session, "saved");
        assert!(matches!(app.input(Event::Resize(80, 24)), Action::None));
    }
    #[test]
    fn streamed_ansi_fragments_are_sanitized_after_delta_reconstruction() {
        let mut app = App::new(true, Arc::new(Mutex::new(UiState::default())));
        app.resize(60, 18);
        app.ingest(&event(
            json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
        ))
        .unwrap();
        for delta in ["\x1b[38;2;", "126;136;142", "mDEMO ready", "\x1b[", "39m"] {
            app.ingest(&event(json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":delta}}))).unwrap();
            app.reconcile().unwrap();
            while app.prepare() {}
            assert!(!plain(&app).contains('['));
        }
        assert!(plain(&app).contains("DEMO ready"));
        assert!(
            app.store.snapshot().messages[0]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains('\x1b')
        );
    }
}
