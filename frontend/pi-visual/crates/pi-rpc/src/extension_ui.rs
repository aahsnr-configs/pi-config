//! Pi's UI subprotocol, never a permission policy. Raw values are retained for replies.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::{Duration, Instant},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Pending,
    Queued,
    Consumed,
    Expired,
    Invalidated,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Select(Vec<String>),
    Confirm(String),
    Input(String),
    Editor(String),
}
#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: String,
    pub title: String,
    pub method: Method,
    pub phase: Phase,
    pub deadline: Option<Instant>,
}
#[derive(Debug, Clone)]
pub enum Answer {
    Value(String),
    Confirmed(bool),
    Cancel,
}
#[derive(Debug)]
pub struct Ticket {
    id: String,
    generation: u64,
    answer: Answer,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiError;
impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid, stale, or unavailable extension UI request")
    }
}
impl std::error::Error for UiError {}
#[derive(Default, Clone)]
pub struct UiState {
    pub dialogs: BTreeMap<String, Dialog>,
    pub status: BTreeMap<String, String>,
    pub widgets: BTreeMap<String, (Vec<String>, String)>,
    pub notifications: VecDeque<(String, String)>,
    pub title: Option<String>,
    pub editor_text: Option<String>,
    seen: BTreeSet<String>,
    generation: u64,
    invalid: bool,
}
fn text(v: &Value, key: &str) -> Result<String, UiError> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| s.len() <= 8192)
        .map(str::to_owned)
        .ok_or(UiError)
}
fn optional(v: &Value, key: &str) -> Result<Option<String>, UiError> {
    if v.get(key).is_none() {
        Ok(None)
    } else {
        text(v, key).map(Some)
    }
}
impl UiState {
    pub fn reset(&mut self) {
        let generation = self.generation.saturating_add(1);
        *self = Self {
            generation,
            ..Self::default()
        };
    }
    pub fn cancel_all(&mut self) {
        self.generation = self.generation.saturating_add(1);
        for d in self.dialogs.values_mut() {
            if matches!(d.phase, Phase::Pending | Phase::Queued) {
                d.phase = Phase::Invalidated;
            }
        }
    }
    pub fn invalidate(&mut self) {
        self.invalid = true;
        for d in self.dialogs.values_mut() {
            if matches!(d.phase, Phase::Pending | Phase::Queued) {
                d.phase = Phase::Invalidated;
            }
        }
    }
    pub fn expire(&mut self, now: Instant) {
        for d in self.dialogs.values_mut() {
            if matches!(d.phase, Phase::Pending | Phase::Queued)
                && d.deadline.is_some_and(|t| now >= t)
            {
                d.phase = Phase::Expired;
            }
        }
    }
    pub fn front(&mut self, now: Instant) -> Option<Dialog> {
        self.expire(now);
        self.dialogs
            .values()
            .find(|d| d.phase == Phase::Pending)
            .cloned()
    }
    pub fn ingest(&mut self, v: &Value, now: Instant) -> Result<(), UiError> {
        let result = self.ingest_inner(v, now);
        if result.is_err() {
            self.invalidate();
        }
        result
    }
    fn ingest_inner(&mut self, v: &Value, now: Instant) -> Result<(), UiError> {
        if self.invalid || v["type"] != "extension_ui_request" {
            return Err(UiError);
        }
        let id = text(v, "id")?;
        if id.is_empty() || id.len() > 256 {
            return Err(UiError);
        }
        let method = text(v, "method")?;
        if ["select", "confirm", "input", "editor"].contains(&method.as_str())
            && (self.seen.len() >= 1024 || !self.seen.insert(id.clone()))
        {
            return Err(UiError);
        }
        let dialog = match method.as_str() {
            "select" => {
                let values = v["options"]
                    .as_array()
                    .filter(|a| !a.is_empty() && a.len() <= 64)
                    .ok_or(UiError)?;
                let mut options = Vec::new();
                let mut shown = BTreeSet::new();
                for x in values {
                    let s = x
                        .as_str()
                        .filter(|s| s.len() <= 8192)
                        .ok_or(UiError)?
                        .to_owned();
                    let visible = crate::reducer::display_text(&s);
                    if visible.trim().is_empty() || !shown.insert(visible) {
                        return Err(UiError);
                    }
                    options.push(s);
                }
                Some(Method::Select(options))
            }
            "confirm" => Some(Method::Confirm(text(v, "message")?)),
            "input" => Some(Method::Input(
                optional(v, "placeholder")?.unwrap_or_default(),
            )),
            "editor" => Some(Method::Editor(optional(v, "prefill")?.unwrap_or_default())),
            "notify" => {
                let kind = optional(v, "notifyType")?.unwrap_or("info".into());
                if !["info", "warning", "error"].contains(&kind.as_str()) {
                    return Err(UiError);
                }
                let message = text(v, "message")?;
                if self.notifications.len() == 32 {
                    self.notifications.pop_front();
                }
                self.notifications.push_back((message, kind));
                None
            }
            "setStatus" => {
                let key = text(v, "statusKey")?;
                match optional(v, "statusText")? {
                    Some(value) => {
                        if self.status.len() >= 32 && !self.status.contains_key(&key) {
                            return Err(UiError);
                        }
                        self.status.insert(key, value);
                    }
                    None => {
                        self.status.remove(&key);
                    }
                }
                None
            }
            "setWidget" => {
                let key = text(v, "widgetKey")?;
                if let Some(lines) = v.get("widgetLines") {
                    let lines = lines
                        .as_array()
                        .filter(|v| v.len() <= 64)
                        .ok_or(UiError)?
                        .iter()
                        .map(|x| {
                            x.as_str()
                                .filter(|s| s.len() <= 8192)
                                .map(str::to_owned)
                                .ok_or(UiError)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let placement = optional(v, "widgetPlacement")?.unwrap_or("aboveEditor".into());
                    if !["aboveEditor", "belowEditor"].contains(&placement.as_str()) {
                        return Err(UiError);
                    }
                    if self.widgets.len() >= 32 && !self.widgets.contains_key(&key) {
                        return Err(UiError);
                    }
                    self.widgets.insert(key, (lines, placement));
                } else {
                    self.widgets.remove(&key);
                }
                None
            }
            "setTitle" => {
                self.title = Some(text(v, "title")?);
                None
            }
            "set_editor_text" => {
                self.editor_text = Some(text(v, "text")?);
                None
            }
            _ => return Err(UiError),
        };
        if let Some(method) = dialog {
            if self
                .dialogs
                .values()
                .filter(|d| matches!(d.phase, Phase::Pending | Phase::Queued))
                .count()
                >= 32
            {
                return Err(UiError);
            }
            let deadline = match v.get("timeout") {
                None => None,
                Some(t) => {
                    let ms = t
                        .as_u64()
                        .filter(|n| *n > 0 && *n <= 86400000)
                        .ok_or(UiError)?;
                    Some(now.checked_add(Duration::from_millis(ms)).ok_or(UiError)?)
                }
            };
            self.dialogs.insert(
                id.clone(),
                Dialog {
                    id,
                    title: text(v, "title")?,
                    method,
                    phase: Phase::Pending,
                    deadline,
                },
            );
        }
        Ok(())
    }
    pub fn queue(&mut self, id: &str, answer: Answer, now: Instant) -> Result<Ticket, UiError> {
        self.expire(now);
        if self.invalid {
            return Err(UiError);
        }
        let d = self
            .dialogs
            .get_mut(id)
            .filter(|d| d.phase == Phase::Pending)
            .ok_or(UiError)?;
        match (&d.method, &answer) {
            (_, Answer::Cancel) => {}
            (Method::Confirm(_), Answer::Confirmed(_)) => {}
            (Method::Select(options), Answer::Value(s)) if options.contains(s) => {}
            (Method::Input(_) | Method::Editor(_), Answer::Value(s)) if s.len() <= 8192 => {}
            _ => return Err(UiError),
        }
        d.phase = Phase::Queued;
        Ok(Ticket {
            id: id.into(),
            generation: self.generation,
            answer,
        })
    }
    pub fn discard(&mut self, t: Ticket) {
        if t.generation == self.generation
            && let Some(d) = self.dialogs.get_mut(&t.id)
            && d.phase == Phase::Queued
        {
            d.phase = Phase::Invalidated;
        }
    }
    pub fn dispatch(&mut self, t: Ticket, now: Instant) -> Result<Value, UiError> {
        self.expire(now);
        if self.invalid || t.generation != self.generation {
            return Err(UiError);
        }
        let d = self
            .dialogs
            .get_mut(&t.id)
            .filter(|d| d.phase == Phase::Queued)
            .ok_or(UiError)?;
        d.phase = Phase::Consumed;
        Ok(match t.answer {
            Answer::Value(value) => json!({"type":"extension_ui_response","id":t.id,"value":value}),
            Answer::Confirmed(confirmed) => {
                json!({"type":"extension_ui_response","id":t.id,"confirmed":confirmed})
            }
            Answer::Cancel => json!({"type":"extension_ui_response","id":t.id,"cancelled":true}),
        })
    }
}
