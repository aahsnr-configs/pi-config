//! Documented session RPC only. Navigation never sends a prompt or runs a tool.
use pi_rpc::client::Handle;
use serde_json::{Value, json};
use std::time::Duration;
const TIMEOUT: Duration = Duration::from_secs(10);
pub struct Loaded {
    pub id: String,
    pub file: Option<String>,
    pub data: Value,
    pub boundary: u64,
    pub discard_through: u64,
    pub preserve_view: bool,
    pub notice: Option<String>,
}
async fn state(h: &Handle) -> Result<Value, String> {
    let r = h
        .request(json!({"type":"get_state"}), TIMEOUT)
        .await
        .map_err(|e| e.to_string())?;
    if !r.success {
        return Err("state request rejected".into());
    }
    r.data
        .filter(|v| v["sessionId"].as_str().is_some())
        .ok_or("missing session identity".into())
}
pub async fn load(h: Handle) -> Result<Loaded, String> {
    let before = state(&h).await?;
    let (entries, boundary) = h
        .request_ordered(json!({"type":"get_entries"}), TIMEOUT)
        .await
        .map_err(|e| e.to_string())?;
    if !entries.success {
        return Err("history request rejected".into());
    }
    let after = state(&h).await?;
    if before["sessionId"] != after["sessionId"] {
        return Err("session changed during history load; reload required".into());
    }
    Ok(Loaded {
        id: before["sessionId"].as_str().unwrap().into(),
        file: after["sessionFile"].as_str().map(str::to_owned),
        data: entries.data.ok_or("missing history")?,
        boundary,
        discard_through: 0,
        preserve_view: false,
        notice: None,
    })
}
pub async fn transition(h: Handle, path: Option<String>) -> Result<Loaded, String> {
    let before = state(&h).await?;
    if before["isStreaming"].as_bool() != Some(false)
        || before["isCompacting"].as_bool() != Some(false)
        || before["pendingMessageCount"].as_u64() != Some(0)
    {
        let mut loaded = load(h).await?;
        loaded.preserve_view = loaded.id == before["sessionId"].as_str().unwrap();
        loaded.notice =
            Some("Session unchanged: stop active or queued work before switching".into());
        return Ok(loaded);
    }
    let command = path.map_or_else(
        || json!({"type":"new_session"}),
        |p| json!({"type":"switch_session","sessionPath":p}),
    );
    let (reply, cutoff) = h
        .request_ordered(command, TIMEOUT)
        .await
        .map_err(|e| e.to_string())?;
    let accepted = reply.success
        && reply
            .data
            .as_ref()
            .is_some_and(|d| d["cancelled"].as_bool() == Some(false));
    let mut loaded = load(h).await?;
    if accepted {
        loaded.discard_through = cutoff;
    } else if loaded.id == before["sessionId"].as_str().unwrap() {
        loaded.preserve_view = true;
        loaded.notice = Some("Session unchanged: backend rejected or vetoed the transition".into());
    } else {
        return Err(
            "backend changed session after a rejected transition; explicit recovery required"
                .into(),
        );
    }
    Ok(loaded)
}
