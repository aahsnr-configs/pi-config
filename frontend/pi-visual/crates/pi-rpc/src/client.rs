//! Owned asynchronous RPC session. Local waiter cancellation never aborts Pi work.
use crate::{Incoming, RpcEvent, RpcResponse, parse_line};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::{mpsc, oneshot, watch},
    task::{JoinHandle, JoinSet},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientError {
    Closed,
    Timeout,
    Capacity,
    Protocol,
    Io,
    InvalidLimits,
}
impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC client: {self:?}")
    }
}
impl std::error::Error for ClientError {}
#[derive(Clone, Copy)]
pub struct Limits {
    pub record_bytes: usize,
    pub event_capacity: usize,
    pub pending_requests: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            record_bytes: 1024 * 1024,
            event_capacity: 32,
            pending_requests: 32,
        }
    }
}
type Reply = oneshot::Sender<Result<(RpcResponse, u64), ClientError>>;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Health {
    pub closed: bool,
    pub desynchronized: bool,
}
enum WriteItem {
    Command(String, Vec<u8>),
    Ui(
        crate::extension_ui::Ticket,
        oneshot::Sender<Result<(), ClientError>>,
    ),
}
struct Shared {
    ui: Arc<Mutex<crate::extension_ui::UiState>>,
    health: watch::Sender<Health>,
    pending: Mutex<HashMap<String, (String, Reply)>>,
    closed: AtomicBool,
    next: AtomicU64,
    stderr: AtomicU64,
    rejected: AtomicU64,
}
impl Shared {
    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.ui.lock().unwrap().invalidate();
        self.health.send_modify(|h| h.closed = true);
        for (_, (_, tx)) in self.pending.lock().unwrap().drain() {
            let _ = tx.send(Err(ClientError::Closed));
        }
    }
}
struct Pending {
    shared: Arc<Shared>,
    id: String,
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.shared.pending.lock().unwrap().remove(&self.id);
    }
}
#[derive(Clone)]
pub struct Handle {
    shared: Arc<Shared>,
    writer: mpsc::Sender<WriteItem>,
    limits: Limits,
}
impl Handle {
    pub fn ui(&self) -> Arc<Mutex<crate::extension_ui::UiState>> {
        self.shared.ui.clone()
    }
    pub async fn respond_ui(
        &self,
        id: &str,
        answer: crate::extension_ui::Answer,
    ) -> Result<(), ClientError> {
        if self.is_closed() || self.shared.health.borrow().desynchronized {
            return Err(ClientError::Closed);
        }
        let ticket = self
            .shared
            .ui
            .lock()
            .unwrap()
            .queue(id, answer, std::time::Instant::now())
            .map_err(|_| ClientError::Protocol)?;
        let (tx, rx) = oneshot::channel();
        if self.writer.try_send(WriteItem::Ui(ticket, tx)).is_err() {
            self.shared.ui.lock().unwrap().invalidate();
            return Err(ClientError::Capacity);
        }
        rx.await.map_err(|_| ClientError::Closed)?
    }
    pub fn is_closed(&self) -> bool {
        self.shared.closed.load(Ordering::Acquire)
    }
    pub fn pending_count(&self) -> usize {
        self.shared.pending.lock().unwrap().len()
    }
    /// Diagnostics contain counts only: never arbitrary stderr or secret-bearing text.
    pub fn diagnostic_bytes(&self) -> u64 {
        self.shared.stderr.load(Ordering::Relaxed)
    }
    pub fn rejected_records(&self) -> u64 {
        self.shared.rejected.load(Ordering::Relaxed)
    }
    pub async fn request(
        &self,
        request: Value,
        timeout: Duration,
    ) -> Result<RpcResponse, ClientError> {
        self.request_ordered(request, timeout).await.map(|(r, _)| r)
    }
    /// Local stdout record boundary, never injected into Pi's wire payload.
    pub async fn request_ordered(
        &self,
        mut request: Value,
        timeout: Duration,
    ) -> Result<(RpcResponse, u64), ClientError> {
        let deadline = tokio::time::Instant::now()
            .checked_add(timeout)
            .filter(|_| !timeout.is_zero())
            .ok_or(ClientError::Timeout)?;
        if self.is_closed() {
            return Err(ClientError::Closed);
        }
        let object = request.as_object_mut().ok_or(ClientError::Protocol)?;
        let command = object
            .get("type")
            .and_then(Value::as_str)
            .ok_or(ClientError::Protocol)?
            .to_owned();
        // Direct shell execution stays disabled pending real permission-route acceptance.
        if command == "bash" || command == "extension_ui_response" {
            return Err(ClientError::Protocol);
        }
        if [
            "abort",
            "abort_retry",
            "new_session",
            "switch_session",
            "fork",
            "navigate_tree",
        ]
        .contains(&command.as_str())
        {
            self.shared.ui.lock().unwrap().cancel_all();
        }
        let n = self
            .shared
            .next
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| ClientError::Capacity)?;
        let id = format!("rpc-{n}");
        object.insert("id".into(), Value::String(id.clone()));
        let mut line = serde_json::to_vec(&request).map_err(|_| ClientError::Protocol)?;
        if line.len() > self.limits.record_bytes {
            return Err(ClientError::Capacity);
        }
        line.push(b'\n');
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.shared.pending.lock().unwrap();
            if self.is_closed() {
                return Err(ClientError::Closed);
            }
            if pending.len() >= self.limits.pending_requests {
                return Err(ClientError::Capacity);
            }
            pending.insert(id.clone(), (command, tx));
        }
        let _guard = Pending {
            shared: self.shared.clone(),
            id: id.clone(),
        };
        self.writer
            .try_send(WriteItem::Command(id, line))
            .map_err(|_| ClientError::Capacity)?;
        tokio::time::timeout_at(deadline, rx)
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(|_| ClientError::Closed)?
    }
}
#[derive(Debug)]
pub struct SequencedEvent {
    pub sequence: u64,
    pub event: RpcEvent,
}
pub struct Client {
    pub ordered_events: Option<mpsc::Receiver<SequencedEvent>>,
    ordered: bool,
    handle: Handle,
    pub events: mpsc::Receiver<RpcEvent>,
    /// Observe alongside events; a protocol gap requires history reconstruction.
    pub health: watch::Receiver<Health>,
    pid: u32,
    stop: Option<oneshot::Sender<()>>,
    supervisor: Option<JoinHandle<()>>,
    limits: Limits,
}
impl Client {
    /// Caller supplies explicit Pi args/cwd/environment. No shell or automatic retry.
    pub fn spawn(command: &mut Command, limits: Limits) -> Result<Self, ClientError> {
        Self::spawn_inner(command, limits, false)
    }
    pub fn spawn_ordered(command: &mut Command, limits: Limits) -> Result<Self, ClientError> {
        Self::spawn_inner(command, limits, true)
    }
    fn spawn_inner(
        command: &mut Command,
        limits: Limits,
        ordered: bool,
    ) -> Result<Self, ClientError> {
        if limits.record_bytes == 0
            || limits.record_bytes > crate::MAX_RPC_LINE_BYTES
            || limits.event_capacity == 0
            || limits.event_capacity > 1024
            || limits.pending_requests == 0
            || limits.pending_requests > 1024
        {
            return Err(ClientError::InvalidLimits);
        }
        use std::process::Stdio;
        command
            .arg("--mode")
            .arg("rpc")
            .process_group(0)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|_| ClientError::Io)?;
        let mut stdin = child.stdin.take().ok_or(ClientError::Io)?;
        let mut stdout = child.stdout.take().ok_or(ClientError::Io)?;
        let mut stderr = child.stderr.take().ok_or(ClientError::Io)?;
        let pid = child.id().ok_or(ClientError::Io)?;
        let (health_tx, health) = watch::channel(Health::default());
        let shared = Arc::new(Shared {
            ui: Arc::new(Mutex::new(crate::extension_ui::UiState::default())),
            health: health_tx,
            pending: Mutex::new(HashMap::new()),
            closed: AtomicBool::new(false),
            next: AtomicU64::new(1),
            stderr: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        });
        let (writer, mut writes) = mpsc::channel::<WriteItem>(limits.pending_requests);
        let (events_tx, events) = mpsc::channel(limits.event_capacity);
        let (ordered_tx, ordered_events) = mpsc::channel(limits.event_capacity);
        let (stop_tx, stop_rx) = oneshot::channel();
        let handle = Handle {
            shared: shared.clone(),
            writer,
            limits,
        };
        let supervisor = tokio::spawn(async move {
            let mut tasks = JoinSet::new();
            let wshared = shared.clone();
            tasks.spawn(async move {
                while let Some(item) = writes.recv().await {
                    let (bytes, ack) = match item {
                        WriteItem::Command(id, bytes) => {
                            if !wshared.pending.lock().unwrap().contains_key(&id) {
                                continue;
                            }
                            (bytes, None)
                        }
                        WriteItem::Ui(ticket, ack) => {
                            if ack.is_closed() {
                                wshared.ui.lock().unwrap().discard(ticket);
                                continue;
                            }
                            if wshared.closed.load(Ordering::Acquire)
                                || wshared.health.borrow().desynchronized
                            {
                                let _ = ack.send(Err(ClientError::Closed));
                                continue;
                            }
                            let value = wshared
                                .ui
                                .lock()
                                .unwrap()
                                .dispatch(ticket, std::time::Instant::now());
                            match value {
                                Ok(value) => {
                                    let mut bytes = serde_json::to_vec(&value).expect("JSON value");
                                    if bytes.len() > limits.record_bytes {
                                        let _ = ack.send(Err(ClientError::Capacity));
                                        continue;
                                    }
                                    bytes.push(b'\n');
                                    (bytes, Some(ack))
                                }
                                Err(_) => {
                                    let _ = ack.send(Err(ClientError::Protocol));
                                    continue;
                                }
                            }
                        }
                    };
                    let result = stdin.write_all(&bytes).await.map_err(|_| ClientError::Io);
                    if let Some(ack) = ack {
                        let _ = ack.send(result);
                    }
                    if result.is_err() {
                        break;
                    }
                }
            });
            let rshared = shared.clone();
            tasks.spawn(async move {
                // This task alone owns framing state; no select branch can discard a fragment.
                let mut bytes = [0u8; 8192];
                let mut line = Vec::new();
                let mut draining = false;
                let mut sequence = 0u64;
                loop {
                    let n = match stdout.read(&mut bytes).await {
                        Ok(0) | Err(_) => {
                            if !line.is_empty() || draining {
                                rshared.ui.lock().unwrap().invalidate();
                                rshared.health.send_modify(|h| h.desynchronized = true);
                            }
                            break;
                        }
                        Ok(n) => n,
                    };
                    for &b in &bytes[..n] {
                        if b != b'\n' {
                            if !draining {
                                if line.len() == limits.record_bytes + 1 {
                                    line.clear();
                                    draining = true;
                                } else {
                                    line.push(b);
                                }
                            }
                            continue;
                        }
                        if line.last() == Some(&b'\r') {
                            line.pop();
                        }
                        let record = if draining || line.len() > limits.record_bytes {
                            None
                        } else {
                            std::str::from_utf8(&line)
                                .ok()
                                .and_then(|s| parse_line(s).ok())
                        };
                        sequence = sequence.saturating_add(1);
                        match record {
                            Some(Incoming::Response(response)) => {
                                if let Some(id) = &response.id
                                    && let Some((command, tx)) =
                                        rshared.pending.lock().unwrap().remove(id)
                                {
                                    let result = if command == response.command {
                                        Ok((response, sequence))
                                    } else {
                                        Err(ClientError::Protocol)
                                    };
                                    let _ = tx.send(result);
                                }
                            }
                            Some(Incoming::Event(event)) => {
                                if event.kind == "agent_settled" {
                                    rshared.ui.lock().unwrap().cancel_all();
                                }
                                if event.kind == "extension_ui_request"
                                    && rshared
                                        .ui
                                        .lock()
                                        .unwrap()
                                        .ingest(&event.payload, std::time::Instant::now())
                                        .is_err()
                                {
                                    rshared.health.send_modify(|h| h.desynchronized = true);
                                }

                                if ordered {
                                    if ordered_tx
                                        .try_send(SequencedEvent { sequence, event })
                                        .is_err()
                                    {
                                        return;
                                    }
                                } else if events_tx.try_send(event).is_err() {
                                    return;
                                }
                            }
                            None => {
                                rshared.rejected.fetch_add(1, Ordering::Relaxed);
                                rshared.ui.lock().unwrap().invalidate();
                                rshared.health.send_modify(|h| h.desynchronized = true);
                            }
                        }
                        line.clear();
                        draining = false;
                    }
                }
            });
            let dshared = shared.clone();
            tasks.spawn(async move {
                let mut bytes = [0u8; 4096];
                loop {
                    match stderr.read(&mut bytes).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let _ = dshared.stderr.try_update(
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                                |v| Some(v.saturating_add(n as u64)),
                            );
                        }
                    }
                }
                std::future::pending::<()>().await;
            });
            tokio::select! {_ = stop_rx=>{},_ = tasks.join_next()=>{}}
            shared.close();
            if let Some(pid) = child
                .id()
                .and_then(|id| rustix::process::Pid::from_raw(id as i32))
            {
                let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
            }
            let _ = child.start_kill();
            let _ = child.wait().await;
            tasks.abort_all();
            while tasks.join_next().await.is_some() {}
        });
        Ok(Self {
            ordered_events: if ordered { Some(ordered_events) } else { None },
            ordered,
            handle,
            events,
            health,
            pid,
            stop: Some(stop_tx),
            supervisor: Some(supervisor),
            limits,
        })
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn handle(&self) -> Handle {
        self.handle.clone()
    }
    pub async fn shutdown(mut self) -> Result<(), ClientError> {
        self.stop_and_join().await
    }
    async fn stop_and_join(&mut self) -> Result<(), ClientError> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.supervisor.as_mut() {
            task.await.map_err(|_| ClientError::Io)?;
        }
        self.supervisor.take();
        Ok(())
    }
    /// Explicit restart never retries old commands; callers must reload history/state.
    pub async fn restart(&mut self, command: &mut Command) -> Result<(), ClientError> {
        self.stop_and_join().await?;
        *self = Self::spawn_inner(command, self.limits, self.ordered)?;
        Ok(())
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}
