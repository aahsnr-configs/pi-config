//! Minimal P03 modal surface; no transcript framework or permission policy.
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use pi_rpc::{
    extension_ui::{Answer, Dialog, Method, UiState},
    reducer::display_text,
};
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
#[derive(Default)]
pub struct Modal {
    pub draft: String,
    pub focus: u64,
    previous: u64,
    pub active: Option<Dialog>,
    pub selected: Option<usize>,
    buffer: Vec<char>,
    cursor: usize,
    pub scroll: u16,
}
impl Modal {
    pub fn sync(&mut self, state: &Arc<Mutex<UiState>>) {
        let mut s = state.lock().unwrap();
        s.expire(Instant::now());
        if self.active.as_ref().is_some_and(|d| {
            s.dialogs
                .get(&d.id)
                .is_none_or(|live| live.phase != pi_rpc::extension_ui::Phase::Pending)
        }) {
            self.active = None;
            self.focus = self.previous;
        }
        if self.active.is_none() {
            if let Some(d) = s.front(Instant::now()) {
                self.previous = self.focus;
                self.focus = 0;
                self.buffer = match &d.method {
                    Method::Editor(s) => s.chars().collect(),
                    _ => Vec::new(),
                };
                self.cursor = self.buffer.len();
                self.selected = None;
                self.scroll = 0;
                self.active = Some(d);
            } else if let Some(text) = s.editor_text.take() {
                if self.draft.is_empty() {
                    self.draft = text;
                } else {
                    s.editor_text = Some(text);
                }
            }
        }
    }
    pub fn paste(&mut self, text: &str) {
        if !self
            .active
            .as_ref()
            .is_some_and(|d| matches!(d.method, Method::Input(_) | Method::Editor(_)))
        {
            return;
        }
        let mut bytes: usize = self.buffer.iter().map(|c| c.len_utf8()).sum();
        for c in display_text(text).chars() {
            if bytes + c.len_utf8() > 8192 {
                break;
            }
            self.buffer.insert(self.cursor, c);
            self.cursor += 1;
            bytes += c.len_utf8();
        }
    }
    pub fn key(&mut self, k: KeyEvent) -> Option<(String, Answer)> {
        if k.kind != KeyEventKind::Press {
            return None;
        }
        let d = self.active.as_ref()?;
        if k.code == KeyCode::Esc
            || (k.code == KeyCode::Char('g') && k.modifiers.contains(KeyModifiers::CONTROL))
        {
            return Some((d.id.clone(), Answer::Cancel));
        }
        if matches!(d.method, Method::Confirm(_) | Method::Select(_)) && !k.modifiers.is_empty() {
            return None;
        }
        match &d.method {
            Method::Confirm(_) => match k.code {
                KeyCode::Char('y') => return Some((d.id.clone(), Answer::Confirmed(true))),
                KeyCode::Char('n') => return Some((d.id.clone(), Answer::Confirmed(false))),
                _ => {}
            },
            Method::Select(options) => match k.code {
                KeyCode::Down => {
                    self.selected =
                        Some(self.selected.map_or(0, |i| (i + 1).min(options.len() - 1)))
                }
                KeyCode::Up => self.selected = Some(self.selected.unwrap_or(1).saturating_sub(1)),
                KeyCode::Enter => {
                    if let Some(i) = self.selected {
                        return Some((d.id.clone(), Answer::Value(options[i].clone())));
                    }
                }
                _ => {}
            },
            Method::Input(_) | Method::Editor(_) => match k.code {
                KeyCode::F(2) => {
                    return Some((d.id.clone(), Answer::Value(self.buffer.iter().collect())));
                }
                KeyCode::Enter => {
                    if matches!(d.method, Method::Input(_))
                        || k.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Some((d.id.clone(), Answer::Value(self.buffer.iter().collect())));
                    }
                    if self.buffer.len() < 8192 {
                        self.buffer.insert(self.cursor, '\n');
                        self.cursor += 1;
                    }
                }
                KeyCode::Char(c)
                    if !k
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                        && self.buffer.len() < 8192 =>
                {
                    self.buffer.insert(self.cursor, c);
                    self.cursor += 1;
                }
                KeyCode::Backspace if self.cursor > 0 => {
                    self.cursor -= 1;
                    self.buffer.remove(self.cursor);
                }
                KeyCode::Delete if self.cursor < self.buffer.len() => {
                    self.buffer.remove(self.cursor);
                }
                KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
                KeyCode::Right => self.cursor = (self.cursor + 1).min(self.buffer.len()),
                KeyCode::Home => self.cursor = 0,
                KeyCode::End => self.cursor = self.buffer.len(),
                _ => {}
            },
        }
        if k.code == KeyCode::PageDown {
            self.scroll = self.scroll.saturating_add(5)
        }
        if k.code == KeyCode::PageUp {
            self.scroll = self.scroll.saturating_sub(5)
        }
        None
    }
    pub fn draw(&self, f: &mut Frame, state: &Arc<Mutex<UiState>>) {
        let a = f.area();
        let s = state.lock().unwrap().clone();
        let mut base = String::from("P03 ISOLATED DIALOG DEMO · Ctrl+C exits\n");
        if let Some(title) = &s.title {
            base.push_str(&format!("Title: {}\n", display_text(title)));
        }
        if let Some(text) = &s.editor_text {
            base.push_str(&format!(
                "Backend editor text staged (Ctrl+R to apply): {}\n",
                display_text(text)
            ));
        }
        for (message, kind) in &s.notifications {
            base.push_str(&format!("{kind}: {}\n", display_text(message)));
        }
        for (key, text) in &s.status {
            base.push_str(&format!("{}: {}\n", display_text(key), display_text(text)));
        }
        for (key, (lines, _)) in &s.widgets {
            base.push_str(&format!(
                "{}: {}\n",
                display_text(key),
                display_text(&lines.join("\n"))
            ));
        }
        base.push_str(&format!("Draft: {}", display_text(&self.draft)));
        f.render_widget(Paragraph::new(base).wrap(Wrap { trim: false }), a);
        self.draw_overlay(f);
    }
    pub fn draw_overlay(&self, f: &mut Frame) {
        let a = f.area();
        if let Some(d) = &self.active {
            let mut body = match &d.method {
                Method::Confirm(m) => format!(
                    "{}\n\ny: Yes · n: No · Esc: cancel (no default)",
                    display_text(m)
                ),
                Method::Select(options) => options
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        format!(
                            "{} {}: {}",
                            if self.selected == Some(i) { ">" } else { " " },
                            i + 1,
                            display_text(v)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                Method::Input(p) => format!(
                    "{}\n{}\nEnter: submit",
                    display_text(p),
                    format_args!(
                        "{}▏{}",
                        display_text(&self.buffer[..self.cursor].iter().collect::<String>()),
                        display_text(&self.buffer[self.cursor..].iter().collect::<String>())
                    )
                ),
                Method::Editor(_) => format!(
                    "{}\nF2 / Ctrl+Enter: submit · Enter: newline",
                    format_args!(
                        "{}▏{}",
                        display_text(&self.buffer[..self.cursor].iter().collect::<String>()),
                        display_text(&self.buffer[self.cursor..].iter().collect::<String>())
                    )
                ),
            };
            body.push_str("\n↑/↓: choose · Esc: cancel · PgUp/PgDn: scroll");
            f.render_widget(Clear, a);
            f.render_widget(
                Paragraph::new(body)
                    .wrap(Wrap { trim: false })
                    .scroll((self.scroll, 0))
                    .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                    .block(
                        Block::default()
                            .title(display_text(&d.title))
                            .borders(Borders::ALL),
                    ),
                a,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn explicit_choice_preserves_raw_value_and_draft_focus() {
        let state = Arc::new(Mutex::new(UiState::default()));
        state.lock().unwrap().ingest(&json!({"type":"extension_ui_request","id":"x","method":"select","title":"Actual\u{1b}]52;fake\u{7}","options":["allow","block\u{1b}[31m"]}),Instant::now()).unwrap();
        let mut m = Modal {
            draft: "draft中".into(),
            focus: 7,
            ..Modal::default()
        };
        m.sync(&state);
        assert!(
            m.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .is_none()
        );
        m.key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        m.key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let (id, answer) = m
            .key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .unwrap();
        assert!(matches!(&answer,Answer::Value(s) if s=="block\u{1b}[31m"));
        let t = state
            .lock()
            .unwrap()
            .queue(&id, answer, Instant::now())
            .unwrap();
        state.lock().unwrap().dispatch(t, Instant::now()).unwrap();
        m.sync(&state);
        assert_eq!(m.focus, 7);
        assert_eq!(m.draft, "draft中");
    }
    #[test]
    fn safe_terminal_and_escape() {
        let state = Arc::new(Mutex::new(UiState::default()));
        state.lock().unwrap().ingest(&json!({"type":"extension_ui_request","id":"x","method":"confirm","title":"\u{1b}]52;bad\u{7}","message":"real"}),Instant::now()).unwrap();
        let mut m = Modal::default();
        m.sync(&state);
        let mut t = ratatui::Terminal::new(ratatui::backend::TestBackend::new(60, 18)).unwrap();
        t.draw(|f| m.draw(f, &state)).unwrap();
        for cell in t.backend().buffer().content() {
            assert!(!cell.symbol().contains('\u{1b}'));
        }
        assert!(matches!(
            m.key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some((_, Answer::Cancel))
        ));
    }
}
#[cfg(test)]
mod shortcut_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn modified_shortcuts_never_confirm() {
        let state = Arc::new(Mutex::new(UiState::default()));
        state.lock().unwrap().ingest(&json!({"type":"extension_ui_request","id":"x","method":"confirm","title":"t","message":"m"}),Instant::now()).unwrap();
        let mut m = Modal::default();
        m.sync(&state);
        for mods in [
            KeyModifiers::CONTROL,
            KeyModifiers::ALT,
            KeyModifiers::SHIFT,
            KeyModifiers::SUPER,
        ] {
            assert!(m.key(KeyEvent::new(KeyCode::Char('y'), mods)).is_none());
        }
        assert!(matches!(
            m.key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
            Some((_, Answer::Confirmed(true)))
        ));
    }
}
