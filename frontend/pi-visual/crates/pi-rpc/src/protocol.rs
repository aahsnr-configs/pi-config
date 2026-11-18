//! Public Pi JSONL envelopes, preserved at crate-root for compatibility.
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

/// A defensive bound on one line (including potential base64 images and tool output).
pub const MAX_RPC_LINE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    EmptyLine,
    LineTooLong,
    InvalidJson(String),
    InvalidEnvelope(&'static str),
    DuplicateRequestId,
    UnmatchedResponseId,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLine => write!(f, "empty RPC line"),
            Self::LineTooLong => write!(f, "RPC line exceeds configured byte limit"),
            Self::InvalidJson(_) => write!(f, "invalid RPC JSON"),
            Self::InvalidEnvelope(field) => write!(f, "invalid RPC envelope: {field}"),
            Self::DuplicateRequestId => write!(f, "duplicate in-flight request ID"),
            Self::UnmatchedResponseId => write!(f, "response has no matching in-flight ID"),
        }
    }
}

impl Error for ProtocolError {}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcResponse {
    pub id: Option<String>,
    pub command: String,
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcEvent {
    pub kind: String,
    /// The complete event is retained to preserve unknown fields and future Pi variants.
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Incoming {
    Response(RpcResponse),
    Event(RpcEvent),
}

/// Parse one complete JSONL record. No model calls, filesystem access or RPC side effects.
pub fn parse_line(line: &str) -> Result<Incoming, ProtocolError> {
    if line.trim().is_empty() {
        return Err(ProtocolError::EmptyLine);
    }
    if line.len() > MAX_RPC_LINE_BYTES {
        return Err(ProtocolError::LineTooLong);
    }
    let value: Value =
        serde_json::from_str(line).map_err(|err| ProtocolError::InvalidJson(err.to_string()))?;
    let object = value
        .as_object()
        .ok_or(ProtocolError::InvalidEnvelope("object required"))?;
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or(ProtocolError::InvalidEnvelope("type"))?
        .to_owned();
    if kind == "response" {
        let command = object
            .get("command")
            .and_then(Value::as_str)
            .ok_or(ProtocolError::InvalidEnvelope("command"))?
            .to_owned();
        let success = object
            .get("success")
            .and_then(Value::as_bool)
            .ok_or(ProtocolError::InvalidEnvelope("success"))?;
        let id = object
            .get("id")
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or(ProtocolError::InvalidEnvelope("id"))
            })
            .transpose()?;
        let error = object
            .get("error")
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or(ProtocolError::InvalidEnvelope("error"))
            })
            .transpose()?;
        Ok(Incoming::Response(RpcResponse {
            id,
            command,
            success,
            data: object.get("data").cloned(),
            error,
            extra: object
                .iter()
                .filter(|(k, _)| {
                    !["type", "id", "command", "success", "data", "error"].contains(&k.as_str())
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }))
    } else {
        Ok(Incoming::Event(RpcEvent {
            kind,
            payload: value,
        }))
    }
}

/// Construct an official read-only request. Requires caller-provided correlation ID.
///
/// Only allowlisted types are accepted so this small preflight helper cannot accidentally
/// launch an agent turn, execute a tool, or mutate user session state.
pub fn read_only_request(id: &str, command: &str) -> Result<Value, ProtocolError> {
    if id.is_empty() {
        return Err(ProtocolError::InvalidEnvelope("nonempty id"));
    }
    const ALLOWED: &[&str] = &[
        "get_state",
        "get_messages",
        "get_entries",
        "get_tree",
        "get_session_stats",
        "get_commands",
    ];
    if !ALLOWED.contains(&command) {
        return Err(ProtocolError::InvalidEnvelope(
            "only read-only requests allowed by this helper",
        ));
    }
    Ok(json!({"id": id, "type": command}))
}

/// Correlate responses by unique request ID independently of arrival order.
#[derive(Debug, Default)]
pub struct RequestTracker {
    pending: HashSet<String>,
}

impl RequestTracker {
    pub fn register(&mut self, id: impl Into<String>) -> Result<(), ProtocolError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ProtocolError::InvalidEnvelope("nonempty id"));
        }
        if !self.pending.insert(id) {
            return Err(ProtocolError::DuplicateRequestId);
        }
        Ok(())
    }

    pub fn complete(&mut self, response: &RpcResponse) -> Result<(), ProtocolError> {
        let id = response
            .id
            .as_deref()
            .ok_or(ProtocolError::UnmatchedResponseId)?;
        if !self.pending.remove(id) {
            return Err(ProtocolError::UnmatchedResponseId);
        }
        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.pending.contains(id)
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Safely encode a complete request or response as a single JSONL line.
pub fn encode_line(value: &Value) -> Result<String, ProtocolError> {
    let line =
        serde_json::to_string(value).map_err(|err| ProtocolError::InvalidJson(err.to_string()))?;
    if line.len() > MAX_RPC_LINE_BYTES {
        return Err(ProtocolError::LineTooLong);
    }
    Ok(format!("{line}\n"))
}

/// Build a named event fixture while preserving arbitrary payload fields.
/// Not used to send protocol commands; intended for offline unit/integration tests.
pub fn fixture_event(kind: &str, fields: Map<String, Value>) -> Value {
    let mut obj = fields;
    obj.insert("type".to_owned(), Value::String(kind.to_owned()));
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_empty_and_invalid_envelopes() {
        assert_eq!(parse_line("\n"), Err(ProtocolError::EmptyLine));
        assert_eq!(
            parse_line("[]"),
            Err(ProtocolError::InvalidEnvelope("object required"))
        );
        assert_eq!(
            parse_line("{}"),
            Err(ProtocolError::InvalidEnvelope("type"))
        );
    }

    #[test]
    fn handles_event_without_losing_future_fields() {
        let input = r#"{"type":"agent_settled","aborted":false,"future":{"ok":true}}"#;
        let Incoming::Event(event) = parse_line(input).expect("valid event") else {
            panic!("expected event");
        };
        assert_eq!(event.kind, "agent_settled");
        assert_eq!(event.payload["future"]["ok"], true);
    }

    #[test]
    fn rejects_unsafe_read_only_request_type() {
        assert!(read_only_request("req-1", "prompt").is_err());
        assert!(read_only_request("req-1", "bash").is_err());
        assert!(read_only_request("req-1", "get_state").is_ok());
    }
}
