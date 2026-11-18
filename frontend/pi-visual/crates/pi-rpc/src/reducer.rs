//! Pure, bounded event reconstruction. Raw text stays untrusted; snapshots are data,
//! not terminal escape sequences. No IO, prompting, or fabricated relationships.
use crate::RpcEvent;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot {
    pub session: Option<String>,
    pub messages: Vec<Value>,
    pub tools: BTreeMap<String, Value>,
    pub entries: BTreeMap<String, Value>,
    pub branch: Vec<String>,
    pub leaf: Option<String>,
    pub settled: bool,
    pub aborted: Option<bool>,
    pub lifecycle: BTreeMap<String, Value>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReduceError;
impl std::fmt::Display for ReduceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid or over-budget workflow event")
    }
}
impl std::error::Error for ReduceError {}
#[derive(Clone)]
pub struct WorkflowStore {
    state: Snapshot,
    active: Option<usize>,
    entry_capacity: usize,
}
impl Default for WorkflowStore {
    fn default() -> Self {
        Self {
            state: Snapshot::default(),
            active: None,
            entry_capacity: 4096,
        }
    }
}
impl WorkflowStore {
    /// Explicit frontend view budget; default remains4096 and the8MiB byte limit
    /// is unchanged. Allows the P04 required10k durable-history stress window.
    pub fn with_entry_capacity(capacity: usize) -> Result<Self, ReduceError> {
        if capacity == 0 || capacity > 10000 {
            return Err(ReduceError);
        }
        Ok(Self {
            entry_capacity: capacity,
            ..Self::default()
        })
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }
    fn commit(&mut self, next: Self) -> Result<(), ReduceError> {
        if next.state.messages.len() > next.entry_capacity
            || next.state.entries.len() > next.entry_capacity
            || next.state.tools.len() > next.entry_capacity
            || serde_json::to_vec(&json!([
                next.state.messages,
                next.state.tools,
                next.state.entries,
                next.state.lifecycle
            ]))
            .map_err(|_| ReduceError)?
            .len()
                > 8 * 1024 * 1024
        {
            return Err(ReduceError);
        }
        *self = next;
        Ok(())
    }
    pub fn apply(&mut self, e: &RpcEvent) -> Result<(), ReduceError> {
        let mut next = self.clone();
        next.apply_inner(e)?;
        self.commit(next)
    }
    fn apply_inner(&mut self, e: &RpcEvent) -> Result<(), ReduceError> {
        let p = &e.payload;
        match e.kind.as_str() {
            "message_start" => {
                let m = p
                    .get("message")
                    .filter(|v| v.is_object())
                    .ok_or(ReduceError)?
                    .clone();
                self.state.messages.push(m);
                self.active = Some(self.state.messages.len() - 1);
            }
            "message_end" => {
                let m = p
                    .get("message")
                    .filter(|v| v.is_object())
                    .ok_or(ReduceError)?
                    .clone();
                if let Some(i) = self.active.take() {
                    self.state.messages[i] = m;
                } else if self.state.messages.last() != Some(&m) {
                    self.state.messages.push(m);
                }
            }
            "message_update" => {
                let i = self.active.ok_or(ReduceError)?;
                let u = p.get("assistantMessageEvent").ok_or(ReduceError)?;
                let t = u["type"].as_str().ok_or(ReduceError)?;
                if let Some(usage) = p.get("usage") {
                    self.state.messages[i]["usage"] = usage.clone();
                }
                if matches!(t, "start" | "done" | "error") {
                    return Ok(());
                }
                if !matches!(
                    t,
                    "text_start"
                        | "text_delta"
                        | "text_end"
                        | "thinking_start"
                        | "thinking_delta"
                        | "thinking_end"
                        | "toolcall_start"
                        | "toolcall_delta"
                        | "toolcall_end"
                ) {
                    return Ok(());
                }
                let index = u["contentIndex"]
                    .as_u64()
                    .filter(|n| *n < 256)
                    .ok_or(ReduceError)? as usize;
                let content = self.state.messages[i]
                    .as_object_mut()
                    .ok_or(ReduceError)?
                    .entry("content")
                    .or_insert(json!([]))
                    .as_array_mut()
                    .ok_or(ReduceError)?;
                while content.len() <= index {
                    content.push(Value::Null);
                }
                let block = &mut content[index];
                match t {
                    "text_start" | "thinking_start" => {
                        let k = if t == "text_start" {
                            "text"
                        } else {
                            "thinking"
                        };
                        *block = json!({"type":k,k:""});
                    }
                    "text_delta" | "thinking_delta" | "toolcall_delta" => {
                        let (kind, key) = match t {
                            "text_delta" => ("text", "text"),
                            "thinking_delta" => ("thinking", "thinking"),
                            _ => ("toolCall", "partialArguments"),
                        };
                        if block.is_null() {
                            *block = json!({"type":kind,key:""});
                        }
                        if block["type"] != kind {
                            return Err(ReduceError);
                        }
                        let delta = u["delta"].as_str().ok_or(ReduceError)?;
                        let mut text = block[key].as_str().unwrap_or("").to_owned();
                        text.push_str(delta);
                        block[key] = Value::String(text);
                    }
                    "text_end" | "thinking_end" => {
                        let k = if t == "text_end" { "text" } else { "thinking" };
                        let value = u["content"].as_str().ok_or(ReduceError)?;
                        *block = json!({"type":k,k:value});
                    }
                    "toolcall_start" => {
                        *block = json!({"type":"toolCall","id":u["id"].as_str().ok_or(ReduceError)?,"name":u["toolName"].as_str().ok_or(ReduceError)?,"partialArguments":""});
                    }
                    "toolcall_end" => {
                        *block = u
                            .get("toolCall")
                            .filter(|v| v.is_object())
                            .ok_or(ReduceError)?
                            .clone();
                    }
                    _ => {}
                }
            }
            "tool_execution_start" | "tool_execution_update" | "tool_execution_end" => {
                let id = p["toolCallId"].as_str().ok_or(ReduceError)?;
                let item = self.state.tools.entry(id.to_owned()).or_insert(json!({}));
                for (k, v) in p.as_object().ok_or(ReduceError)? {
                    item[k] = v.clone();
                }
            }
            "agent_start" => {
                self.state.settled = false;
                self.state.aborted = None;
                self.active = None;
                self.state.lifecycle.insert(e.kind.clone(), p.clone());
            }
            "agent_settled" => {
                self.state.settled = true;
                self.state.aborted = Some(p["aborted"].as_bool().ok_or(ReduceError)?);
                self.state.lifecycle.insert(e.kind.clone(), p.clone());
            }
            "agent_end"
            | "turn_start"
            | "turn_end"
            | "auto_retry_start"
            | "auto_retry_end"
            | "compaction_start"
            | "compaction_end"
            | "summarization_retry_scheduled"
            | "summarization_retry_attempt_start"
            | "summarization_retry_finished"
            | "queue_update" => {
                self.state.lifecycle.insert(e.kind.clone(), p.clone());
            }
            "entry_appended" => {
                let entry = p.get("entry").ok_or(ReduceError)?;
                let id = entry["id"].as_str().ok_or(ReduceError)?;
                if self.state.entries.get(id).is_some_and(|v| v != entry) {
                    return Err(ReduceError);
                }
                self.state.entries.insert(id.into(), entry.clone());
            }
            _ => {}
        }
        Ok(())
    }
    /// Apply a get_entries payload. Incremental cursors belong to exactly one session.
    pub fn restore(
        &mut self,
        session: &str,
        data: &Value,
        incremental: bool,
    ) -> Result<(), ReduceError> {
        let mut next = if incremental {
            if self.state.session.as_deref() != Some(session) {
                return Err(ReduceError);
            }
            self.clone()
        } else {
            Self {
                entry_capacity: self.entry_capacity,
                ..Self::default()
            }
        };
        next.state.session = Some(session.into());
        for e in data["entries"].as_array().ok_or(ReduceError)? {
            let id = e["id"].as_str().ok_or(ReduceError)?;
            if next.state.entries.get(id).is_some_and(|old| old != e) {
                return Err(ReduceError);
            }
            next.state.entries.insert(id.into(), e.clone());
        }
        let leaf = data.get("leafId").ok_or(ReduceError)?;
        next.state.leaf = if leaf.is_null() {
            None
        } else {
            Some(leaf.as_str().ok_or(ReduceError)?.into())
        };
        next.state.branch.clear();
        let mut cursor = next.state.leaf.clone();
        let mut seen = BTreeSet::new();
        while let Some(id) = cursor {
            if !seen.insert(id.clone()) {
                return Err(ReduceError);
            }
            let e = next.state.entries.get(&id).ok_or(ReduceError)?;
            cursor = match e.get("parentId") {
                Some(Value::Null) => None,
                Some(Value::String(s)) => Some(s.clone()),
                _ => return Err(ReduceError),
            };
            next.state.branch.push(id);
        }
        next.state.branch.reverse();
        next.state.messages = next
            .state
            .branch
            .iter()
            .filter_map(|id| next.state.entries[id].get("message").cloned())
            .collect();
        next.active = None;
        next.state.tools.clear();
        next.state.lifecycle.clear();
        next.state.settled = false;
        next.state.aborted = None;
        self.commit(next)
    }
    /// Flatten documented get_tree nodes without recursive traversal.
    pub fn restore_tree(&mut self, session: &str, data: &Value) -> Result<(), ReduceError> {
        let mut stack: Vec<&Value> = data["tree"].as_array().ok_or(ReduceError)?.iter().collect();
        let mut entries = Vec::new();
        while let Some(node) = stack.pop() {
            if entries.len() >= self.entry_capacity {
                return Err(ReduceError);
            }
            entries.push(node.get("entry").ok_or(ReduceError)?.clone());
            stack.extend(node["children"].as_array().ok_or(ReduceError)?);
        }
        self.restore(
            session,
            &json!({"entries":entries,"leafId":data["leafId"]}),
            false,
        )
    }
}
/// Display-only sanitization. Never mutate persisted/raw message data.
pub fn display_text(raw: &str) -> String {
    crate::terminal_text::plain(raw)
}
