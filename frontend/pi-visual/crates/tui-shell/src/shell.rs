//! P04 runtime: owns input, rendering and existing RPC client; offline has no client.
use crate::{
    app::{Action, App},
    terminal::{Input, TerminalGuard},
    transcript::Entry,
};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pi_rpc::{Incoming, RpcEvent, client::Client};
use serde_json::{Value, json};
use std::{
    io,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    signal::unix::{SignalKind, signal},
    task::JoinSet,
};
const FIXTURE: &str = include_str!("../../../tests/fixtures/p04-transcript.jsonl");
fn parse_replay(text: &str) -> io::Result<Vec<Incoming>> {
    if text.len() > 16 * 1024 * 1024 {
        return Err(io::Error::other("replay exceeds 16 MiB"));
    }
    let mut events = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if events.len() >= 100000 {
            return Err(io::Error::other("replay event budget exceeded"));
        }
        events.push(pi_rpc::parse_line(line).map_err(io::Error::other)?);
    }
    Ok(events)
}
fn replay_source(path: &Path) -> io::Result<String> {
    use std::io::Read;
    use std::os::fd::AsRawFd;
    fn permitted(path: &Path) -> bool {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        !["auth.json", "settings.json", "credentials", ".bash_history"].contains(&name)
            && !name.starts_with(".env")
            && !["pem", "key", "p12"]
                .contains(&path.extension().and_then(|s| s.to_str()).unwrap_or(""))
            && !path.components().any(|c| {
                [".ssh", ".aws", ".gnupg"]
                    .iter()
                    .any(|s| c.as_os_str() == *s)
            })
            && (!path
                .components()
                .map(|c| c.as_os_str())
                .collect::<Vec<_>>()
                .windows(2)
                .any(|p| p[0] == ".pi" && p[1] == "agent")
                || path.starts_with(project))
    }
    let path = path.canonicalize()?;
    if !permitted(&path) {
        return Err(io::Error::other(
            "sensitive/configuration paths are not replay inputs",
        ));
    }
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
        .open(&path)?;
    let actual = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    if !permitted(&actual) {
        return Err(io::Error::other("opened replay target is not permitted"));
    }
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
        return Err(io::Error::other("replay must be a regular file <=16 MiB"));
    }
    let mut source = String::new();
    file.take(16 * 1024 * 1024 + 1)
        .read_to_string(&mut source)?;
    if source.len() > 16 * 1024 * 1024 {
        return Err(io::Error::other("replay grew beyond byte budget"));
    }
    Ok(source)
}
pub fn replay_file(path: &str) -> io::Result<()> {
    run_offline(&replay_source(Path::new(path))?)
}
pub fn long_demo() -> io::Result<()> {
    let mut app = App::new(true, Arc::new(Mutex::new(Default::default())));
    app.status = "DEMO stress: 10,000 synthetic entries · no Pi process".into();
    app.transcript.replace((0..10000).map(|i|Entry{id:format!("stress-{i}"),title:format!("DEMO assistant {i}"),raw:"Synthetic Markdown **stress fixture** with 中 and 🙂.\n\nNo actual backend action.".into(),markdown:true,color:ratatui::style::Color::Cyan,revision:0}).collect()).map_err(io::Error::other)?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(app, None, None))
}
pub fn demo() -> io::Result<()> {
    run_offline(FIXTURE)
}
fn run_offline(source: &str) -> io::Result<()> {
    let events = parse_replay(source)?;
    let mut app = App::new(true, Arc::new(Mutex::new(Default::default())));
    let mut session = "replay".to_string();
    for record in events {
        match record {
            Incoming::Event(e) => app.ingest(&e).map_err(io::Error::other)?,
            Incoming::Response(r) if r.success && r.command == "get_state" => {
                if let Some(id) = r.data.as_ref().and_then(|v| v["sessionId"].as_str()) {
                    session = id.into();
                }
            }
            Incoming::Response(r) if r.success && r.command == "get_entries" => app
                .restore(
                    &session,
                    &r.data
                        .ok_or_else(|| io::Error::other("missing replay history"))?,
                )
                .map_err(io::Error::other)?,
            _ => {}
        }
    }
    app.reconcile().map_err(io::Error::other)?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(app, None, None))
}
/// Accepts an already-created P02 client; never creates another backend for rendering.
pub async fn run_live(client: Client) -> io::Result<()> {
    let mut app = App::new(false, client.handle().ui());
    app.status = "RPC loading history".into();
    run(app, Some(client), None).await
}
pub async fn start(launch: crate::launch::Launch) -> io::Result<()> {
    let client = launch.spawn()?;
    let mut app = App::new(false, client.handle().ui());
    app.set_directory(&launch.cwd);
    app.status = "RPC loading history".into();
    run(app, Some(client), Some(launch)).await
}
#[derive(Clone, Copy)]
enum JobKind {
    Prompt,
    History(u64),
    Other,
}
enum Completion {
    Accepted(String),
    Restore(crate::session::Loaded),
    Done,
}
type Job = (JobKind, Result<Completion, String>);
struct Resources {
    client: Option<Client>,
    terminal: Option<TerminalGuard>,
    input: Option<Input>,
    tasks: JoinSet<Job>,
}
fn job<F>(tasks: &mut JoinSet<Job>, kind: JobKind, future: F)
where
    F: std::future::Future<Output = Result<Completion, String>> + Send + 'static,
{
    tasks.spawn(async move { (kind, future.await) });
}
async fn history(h: pi_rpc::client::Handle) -> Result<Completion, String> {
    crate::session::load(h).await.map(Completion::Restore)
}
#[derive(Default)]
pub(crate) struct HistoryBuffer {
    events: Vec<pi_rpc::client::SequencedEvent>,
    bytes: usize,
}
impl HistoryBuffer {
    pub(crate) fn push(&mut self, e: pi_rpc::client::SequencedEvent) -> io::Result<()> {
        self.bytes += serde_json::to_vec(&e.event.payload)?.len();
        if self.bytes > 16 * 1024 * 1024 || self.events.len() >= 1024 {
            return Err(io::Error::other(
                "history/live reconciliation budget exceeded",
            ));
        }
        self.events.push(e);
        Ok(())
    }
    pub(crate) fn apply_loaded(
        &mut self,
        app: &mut App,
        loaded: &crate::session::Loaded,
    ) -> io::Result<()> {
        self.events.retain(|e| e.sequence > loaded.discard_through);
        if loaded.preserve_view {
            for e in self.events.drain(..) {
                app.ingest(&e.event).map_err(io::Error::other)?;
            }
            self.bytes = 0;
            app.reconcile().map_err(io::Error::other)
        } else {
            self.restore(app, &loaded.id, &loaded.data, loaded.boundary)
        }
    }
    fn restore(&mut self, app: &mut App, id: &str, data: &Value, boundary: u64) -> io::Result<()> {
        let mut active_message = None;
        let mut active_tools = std::collections::BTreeSet::new();
        let mut agent_start = None;
        for (i, e) in self
            .events
            .iter()
            .enumerate()
            .take_while(|(_, e)| e.sequence <= boundary)
        {
            match e.event.kind.as_str() {
                "agent_start" => agent_start = Some(i),
                "message_start" => active_message = Some(i),
                "message_end" => active_message = None,
                "tool_execution_start" => {
                    if let Some(id) = e.event.payload["toolCallId"].as_str() {
                        active_tools.insert(id.to_string());
                    }
                }
                "tool_execution_end" => {
                    if let Some(id) = e.event.payload["toolCallId"].as_str() {
                        active_tools.remove(id);
                    }
                }
                _ => {}
            }
        }
        app.restore(id, data).map_err(io::Error::other)?;
        for (i, e) in self.events.drain(..).enumerate() {
            let live = e.sequence > boundary;
            let partial = active_message.is_some_and(|start| i >= start)
                && matches!(
                    e.event.kind.as_str(),
                    "message_start" | "message_update" | "message_end"
                );
            let tool = e.event.kind.starts_with("tool_execution_")
                && e.event.payload["toolCallId"]
                    .as_str()
                    .is_some_and(|id| active_tools.contains(id));
            let agent =
                agent_start == Some(i) && (active_message.is_some() || !active_tools.is_empty());
            if live || partial || tool || agent {
                app.ingest(&e.event).map_err(io::Error::other)?;
            }
        }
        self.bytes = 0;
        app.reconcile().map_err(io::Error::other)
    }
}
fn finish_restoration(
    app: &mut App,
    client: Option<&Client>,
    active: bool,
    notice: Option<String>,
) {
    // Generation-tagged jobs and abort+join before restart prevent old completions
    // crossing clients. A completion from this client can still race its EOF.
    let healthy = client.is_some_and(|c| {
        let health = *c.health.borrow();
        !health.closed && !health.desynchronized && !c.handle().is_closed()
    });
    if active && healthy {
        app.connected = true;
        app.status = notice
            .unwrap_or_else(|| format!("RPC ready · session {} · history restored", app.session));
    } else {
        app.disconnected();
    }
}
enum Source {
    Input(Option<io::Result<Event>>),
    Rpc(Option<pi_rpc::client::SequencedEvent>),
    Health,
    Tick,
    Stop,
    Job(Option<Result<Job, tokio::task::JoinError>>),
}
async fn run(
    app: App,
    client: Option<Client>,
    launch: Option<crate::launch::Launch>,
) -> io::Result<()> {
    owned(client, move |r| Box::pin(run_inner(app, r, launch))).await
}
async fn owned<F>(client: Option<Client>, body: F) -> io::Result<()>
where
    F: for<'a> FnOnce(
        &'a mut Resources,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = io::Result<()>> + 'a>>,
{
    let mut r = Resources {
        client,
        terminal: None,
        input: None,
        tasks: JoinSet::new(),
    };
    let result = {
        let mut future = body(&mut r);
        std::future::poll_fn(|cx| {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                future.as_mut().poll(cx)
            })) {
                Ok(poll) => poll,
                Err(_) => std::task::Poll::Ready(Err(io::Error::other(
                    "frontend loop panicked; resources restored",
                ))),
            }
        })
        .await
    };
    r.tasks.abort_all();
    while r.tasks.join_next().await.is_some() {}
    if let Some(input) = r.input.as_mut() {
        input.shutdown().await;
    }
    let close = if let Some(c) = r.client.take() {
        c.shutdown().await.map_err(io::Error::other)
    } else {
        Ok(())
    };
    drop(r.terminal.take());
    result.and(close)
}
async fn run_inner(
    mut app: App,
    r: &mut Resources,
    launch: Option<crate::launch::Launch>,
) -> io::Result<()> {
    let mut term = signal(SignalKind::terminate())?;
    let mut hup = signal(SignalKind::hangup())?;
    let mut interrupt = signal(SignalKind::interrupt())?;
    r.terminal = Some(TerminalGuard::enter()?);
    r.input = Some(Input::start());
    let guard = r.terminal.as_mut().unwrap();
    let input = r.input.as_mut().unwrap();
    let client = &mut r.client;
    let tasks = &mut r.tasks;
    let mut connected = client.is_some();
    let mut generation = 0u64;
    let mut navigation_draft: Option<String> = None;
    let mut session_file: Option<String> = None;
    let mut expected_restart: Option<String> = None;
    let mut prompt_pending = false;
    let mut hydrating = connected;
    let mut buffer = HistoryBuffer::default();
    let mut demo_dialog_count = 0u64;
    let mut demo_stream = std::collections::VecDeque::new();
    demo_stream
        .push_back(json!({"type":"message_start","message":{"role":"assistant","content":[]}}));
    for n in 0..80 {
        demo_stream.push_back(json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":format!("DEMO streamed chunk {n}. ")}}));
    }
    if let Some(c) = client.as_ref() {
        if c.ordered_events.is_none() {
            return Err(io::Error::other(
                "live frontend requires Client::spawn_ordered for history fencing",
            ));
        }
        job(tasks, JobKind::History(generation), history(c.handle()));
    }
    loop {
        let size = guard.terminal.size()?;
        if app.transcript.width != size.width as usize
            || app.transcript.height != size.height.saturating_sub(5) as usize
        {
            app.resize(size.width, size.height);
        }
        if app.dirty || app.loading {
            app.prepare();
            guard.terminal.draw(|f| app.draw(f))?;
            app.dirty = false;
        }
        let tick = async {
            if app.loading || app.modal.active.is_some() {
                tokio::time::sleep(Duration::from_millis(20)).await
            } else {
                std::future::pending::<()>().await
            }
        };
        let source = if let Some(c) = client.as_mut().filter(|_| connected) {
            tokio::select! {
             event=input.events.recv()=>Source::Input(event),e=c.ordered_events.as_mut().unwrap().recv()=>Source::Rpc(e),_=c.health.changed()=>Source::Health,_=tick=>Source::Tick,done=tasks.join_next(),if !tasks.is_empty()=>Source::Job(done),_=term.recv()=>Source::Stop,_=hup.recv()=>Source::Stop,_=interrupt.recv()=>Source::Stop
            }
        } else {
            tokio::select! {event=input.events.recv()=>Source::Input(event),_=tick=>Source::Tick,done=tasks.join_next(),if !tasks.is_empty()=>Source::Job(done),_=term.recv()=>Source::Stop,_=hup.recv()=>Source::Stop,_=interrupt.recv()=>Source::Stop}
        };
        match source {
            Source::Stop | Source::Input(None) => break,
            Source::Input(Some(Err(e))) => return Err(e),
            Source::Input(Some(Ok(event))) => {
                if app.offline && app.modal.active.is_none() {
                    if matches!(&event,Event::Key(k) if k.kind==KeyEventKind::Press&&k.code==KeyCode::F(4))
                    {
                        demo_dialog_count += 1;
                        let event = RpcEvent {
                            kind: "extension_ui_request".into(),
                            payload: json!({"type":"extension_ui_request","id":format!("demo-{demo_dialog_count}"),"method":"confirm","title":"DEMO permission interruption · local replay only","message":"No host action or RPC reply follows.","timeout":10000}),
                        };
                        app.ingest(&event).map_err(io::Error::other)?;
                        continue;
                    }
                    if app.transcript.entries.len() < 10000
                        && matches!(&event,Event::Key(k) if k.kind==KeyEventKind::Press&&k.code==KeyCode::F(2))
                    {
                        if let Some(payload) = demo_stream.pop_front() {
                            let event = RpcEvent {
                                kind: payload["type"].as_str().unwrap().into(),
                                payload,
                            };
                            app.ingest(&event).map_err(io::Error::other)?;
                            app.reconcile().map_err(io::Error::other)?;
                        }
                        continue;
                    }
                }
                match app.input(event) {
                    Action::Exit => break,
                    Action::Restart => {
                        if hydrating {
                            app.status =
                                "Wait for the current session transition before restarting".into();
                            continue;
                        }
                        if let (Some(launch), Some(c)) = (launch.as_ref(), client.as_mut()) {
                            if session_file.is_none() && !app.store.snapshot().messages.is_empty() {
                                app.status =
                                    "Cannot recover an unsaved session; draft/transcript retained"
                                        .into();
                                continue;
                            }
                            tasks.abort_all();
                            while tasks.join_next().await.is_some() {}
                            app.ui.lock().unwrap().invalidate();
                            app.modal.sync(&app.ui);
                            expected_restart = session_file.as_ref().map(|_| app.session.clone());
                            match c
                                .restart(&mut launch.command(session_file.as_deref()))
                                .await
                            {
                                Ok(()) => {
                                    app.ui = c.handle().ui();
                                    app.connected = true;
                                    generation += 1;
                                    connected = true;
                                    hydrating = true;
                                    prompt_pending = false;
                                    buffer = HistoryBuffer::default();
                                    app.status = "RPC restarting · restoring saved session".into();
                                    job(tasks, JobKind::History(generation), history(c.handle()));
                                }
                                Err(error) => {
                                    connected = false;
                                    app.disconnected();
                                    app.status = format!("RPC restart failed: {error}");
                                }
                            }
                        } else {
                            app.status = "Restart unavailable for this fixture attachment".into();
                        }
                    }
                    Action::None => {}
                    Action::Reply(id, answer) => {
                        if let Some(c) = client.as_ref() {
                            if tasks.len() < 8 {
                                let h = c.handle();
                                job(tasks, JobKind::Other, async move {
                                    h.respond_ui(&id, answer).await.map_err(|e| e.to_string())?;
                                    Ok(Completion::Done)
                                });
                            }
                        } else {
                            let mut ui = app.ui.lock().unwrap();
                            let t = ui
                                .queue(&id, answer, std::time::Instant::now())
                                .map_err(io::Error::other)?;
                            ui.dispatch(t, std::time::Instant::now())
                                .map_err(io::Error::other)?;
                            app.status = "DEMO: answer kept local; zero RPC calls".into();
                        }
                    }
                    Action::Abort => {
                        if let Some(c) = client.as_ref().filter(|_| connected)
                            && tasks.len() < 8
                        {
                            let h = c.handle();
                            job(tasks, JobKind::Other, async move {
                                h.request(json!({"type":"abort"}), Duration::from_secs(5))
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(Completion::Done)
                            });
                        }
                    }
                    Action::Prompt(text) => {
                        if let Some(c) = client.as_ref().filter(|_| connected)
                            && tasks.len() < 8
                            && !prompt_pending
                            && !hydrating
                        {
                            let h = c.handle();
                            let switching = text == "/new" || text.starts_with("/resume ");
                            if switching {
                                generation += 1;
                                navigation_draft = Some(text.clone());
                                hydrating = true;
                                buffer = HistoryBuffer::default();
                                app.status = "RPC switching session · draft retained".into();
                                job(tasks, JobKind::History(generation), async move {
                                    let path = if text == "/new" {
                                        None
                                    } else {
                                        Some(text.trim_start_matches("/resume ").into())
                                    };
                                    crate::session::transition(h, path)
                                        .await
                                        .map(Completion::Restore)
                                });
                            } else {
                                prompt_pending = true;
                                job(tasks, JobKind::Prompt, async move {
                                    let response = h
                                        .request(
                                            json!({"type":"prompt","message":text}),
                                            Duration::from_secs(30),
                                        )
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    if !response.success {
                                        return Err("prompt rejected; draft retained".into());
                                    }
                                    Ok(Completion::Accepted(text))
                                });
                            }
                        }
                    }
                }
            }
            Source::Rpc(Some(e)) => {
                if hydrating {
                    buffer.push(e)?;
                    app.dirty = true;
                } else {
                    app.ingest(&e.event).map_err(io::Error::other)?;
                }
                if let Some(c) = client.as_mut() {
                    for _ in 0..31 {
                        match c.ordered_events.as_mut().unwrap().try_recv() {
                            Ok(e) => {
                                if hydrating {
                                    buffer.push(e)?;
                                } else {
                                    app.ingest(&e.event).map_err(io::Error::other)?;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                if !hydrating {
                    app.reconcile().map_err(io::Error::other)?;
                }
            }
            Source::Rpc(None) => {
                connected = false;
                app.disconnected();
            }
            Source::Health => {
                if let Some(c) = client.as_ref() {
                    let health = *c.health.borrow();
                    if health.closed || health.desynchronized {
                        connected = false;
                        app.disconnected();
                    }
                }
            }
            Source::Tick => {
                let before = app.modal.active.as_ref().map(|d| d.id.clone());
                app.modal.sync(&app.ui);
                if before != app.modal.active.as_ref().map(|d| d.id.clone()) {
                    app.dirty = true;
                }
            }
            Source::Job(Some(result)) => {
                let (kind, result) = result.map_err(io::Error::other)?;
                if matches!(kind,JobKind::History(expected) if expected != generation) {
                    continue;
                }
                match result {
                    Ok(Completion::Accepted(text)) => {
                        if app.modal.draft == text {
                            app.modal.draft.clear();
                        }
                        prompt_pending = false;
                    }
                    Ok(Completion::Restore(loaded)) => {
                        if expected_restart.take().is_some_and(|id| id != loaded.id) {
                            return Err(io::Error::other("RPC recovery session identity mismatch"));
                        }
                        if let Some(c) = client.as_mut() {
                            while let Ok(e) = c.ordered_events.as_mut().unwrap().try_recv() {
                                buffer.push(e)?;
                            }
                        }
                        buffer.apply_loaded(&mut app, &loaded)?;
                        if let Some(submitted) = navigation_draft.take()
                            && !loaded.preserve_view
                            && app.modal.draft == submitted
                        {
                            app.modal.draft.clear();
                        }
                        session_file = loaded.file;
                        finish_restoration(&mut app, client.as_ref(), connected, loaded.notice);
                        connected = app.connected;
                        hydrating = false;
                    }
                    Ok(Completion::Done) => {}
                    Err(error) => {
                        match kind {
                            JobKind::Prompt => prompt_pending = false,
                            JobKind::History(_) => {
                                hydrating = false;
                                connected = false;
                                app.disconnected();
                            }
                            JobKind::Other => {}
                        }
                        app.status = if connected {
                            format!("RPC error: {error}")
                        } else {
                            format!("RPC disconnected · {error}")
                        };
                    }
                }
                app.dirty = true;
            }
            Source::Job(None) => {}
        }
    }
    app.ui.lock().unwrap().invalidate();
    Ok(())
}

pub fn benchmark() {
    benchmark_ingestion();
    for n in [100, 1000, 10000] {
        let mut t = crate::transcript::Transcript::new();
        t.replace((0..n).map(|i|Entry{id:format!("bench-{i}"),title:"DEMO assistant".into(),raw:"# Heading\nA fixed **Markdown** paragraph with 中 and 🙂.\n\n```rust\nlet x = 1;\n```".into(),markdown:true,color:ratatui::style::Color::Cyan,revision:0}).collect()).unwrap();
        t.resize(100, 25);
        let cold = std::time::Instant::now();
        while t.measure_batch(32) {}
        let cold = cold.elapsed();
        let warm = std::time::Instant::now();
        for _ in 0..100 {
            t.visible();
        }
        let warm_us = warm.elapsed().as_micros();
        let painted = t.painted;
        let reflow = std::time::Instant::now();
        t.resize(80, 25);
        while t.measure_batch(32) {}
        let reflow_us = reflow.elapsed().as_micros();
        println!(
            "DEMO n={n}: cold_layout_us={} warm_100_viewports_us={warm_us} reflow_us={reflow_us} visible_blocks={painted} formatted_total={}",
            cold.as_micros(),
            t.formatted
        );
    }
}
fn benchmark_ingestion() {
    for count in [100, 1000] {
        let mut store = pi_rpc::reducer::WorkflowStore::default();
        let start = std::time::Instant::now();
        for i in 0..count {
            let m = json!({"role":"user","content":format!("DEMO message {i}")});
            for kind in ["message_start", "message_end"] {
                store
                    .apply(&RpcEvent {
                        kind: kind.into(),
                        payload: json!({"type":kind,"message":m}),
                    })
                    .unwrap();
            }
        }
        println!(
            "DEMO P02 ingestion messages={count} events={} elapsed_us={} (separate from painting)",
            count * 2,
            start.elapsed().as_micros()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_replay_rejected() {
        assert!(parse_replay("invalid\n").is_err());
        assert!(parse_replay(FIXTURE).is_ok());
    }
}
#[cfg(test)]
mod regression_tests {
    use super::*;
    #[tokio::test]
    async fn delayed_history_keeps_partial_prefix_and_later_events_without_duplicates() {
        let mut command = tokio::process::Command::new("python3");
        command.env_clear().args(["-u","-c",r#"import json,sys
r=json.loads(sys.stdin.readline());print(json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True,data=dict(sessionId='s'))))
r=json.loads(sys.stdin.readline())
u=dict(role='user',content='saved')
for t in ['message_start','message_end']:print(json.dumps(dict(type=t,message=u)))
print(json.dumps(dict(type='message_start',message=dict(role='assistant',content=[]))))
print(json.dumps(dict(type='message_update',assistantMessageEvent=dict(type='text_delta',contentIndex=0,delta='prefix '))))
print(json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True,data=dict(entries=[dict(id='u',parentId=None,type='message',message=u)],leafId='u'))))
print(json.dumps(dict(type='message_update',assistantMessageEvent=dict(type='text_delta',contentIndex=0,delta='suffix'))))
print(json.dumps(dict(type='message_end',message=dict(role='assistant',content=[dict(type='text',text='prefix suffix')]))))
r=json.loads(sys.stdin.readline());print(json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True,data=dict(sessionId='s'))))
sys.stdin.read()"#]);
        let mut client = Client::spawn_ordered(&mut command, Default::default()).unwrap();
        let Completion::Restore(loaded) = history(client.handle()).await.unwrap() else {
            panic!()
        };
        let mut buffer = HistoryBuffer::default();
        for _ in 0..6 {
            buffer
                .push(
                    client
                        .ordered_events
                        .as_mut()
                        .unwrap()
                        .recv()
                        .await
                        .unwrap(),
                )
                .unwrap();
        }
        let mut app = App::new(false, client.handle().ui());
        buffer.apply_loaded(&mut app, &loaded).unwrap();
        let snapshot = app.store.snapshot();
        assert_eq!(snapshot.messages.len(), 2);
        assert_eq!(snapshot.messages[0]["content"], "saved");
        assert_eq!(snapshot.messages[1]["content"][0]["text"], "prefix suffix");
        client.shutdown().await.unwrap();
    }
    #[tokio::test]
    async fn history_completion_after_eof_cannot_reenable_submission() {
        let mut command = tokio::process::Command::new("python3");
        command.env_clear().args(["-u","-c",r#"import json,sys
for i in range(3):
 r=json.loads(sys.stdin.readline())
 data=dict(entries=[dict(type='message',id='saved',parentId=None,message=dict(role='user',content='saved history'))],leafId='saved') if r['type']=='get_entries' else dict(sessionId='closed-session')
 print(json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True,data=data)),flush=True)
"#]);
        let mut client = Client::spawn_ordered(&mut command, Default::default()).unwrap();
        let loaded = crate::session::load(client.handle()).await.unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            while !client.handle().is_closed() {
                client.health.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let mut app = App::new(false, client.handle().ui());
        app.modal.draft = "must stay unsent".into();
        HistoryBuffer::default()
            .apply_loaded(&mut app, &loaded)
            .unwrap();
        finish_restoration(&mut app, Some(&client), true, loaded.notice);
        assert_eq!(app.session, "closed-session");
        assert_eq!(app.transcript.entries.len(), 1);
        assert!(!app.connected);
        assert!(app.status.starts_with("RPC disconnected"));
        assert!(matches!(
            app.input(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Enter,
                crossterm::event::KeyModifiers::NONE
            ))),
            Action::None
        ));
        assert_eq!(app.modal.draft, "must stay unsent");
        client.shutdown().await.unwrap();
    }
    #[test]
    #[ignore = "requires PTY; lifecycle harness runs this with child and reader active"]
    fn panic_with_child_and_reader_is_contained_and_joined() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let mut command = tokio::process::Command::new("python3");
            command
                .env_clear()
                .args(["-u", "-c", "import time;time.sleep(60)"]);
            let client = Client::spawn(&mut command, Default::default()).unwrap();
            let pid = client.pid();
            let result = owned(Some(client), |r| {
                Box::pin(async move {
                    r.terminal = Some(TerminalGuard::enter()?);
                    r.input = Some(Input::start());
                    let h = r.client.as_ref().unwrap().handle();
                    job(&mut r.tasks, JobKind::Other, async move {
                        h.request(json!({"type":"get_state"}), Duration::from_secs(60))
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok(Completion::Done)
                    });
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    panic!("controlled session panic")
                })
            })
            .await;
            assert!(result.is_err());
            assert!(!Path::new(&format!("/proc/{pid}")).exists());
            assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap());
        });
    }
}

#[cfg(test)]
mod transition_tests {
    use super::*;
    #[test]
    fn switch_fence_excludes_old_partial_messages_and_tools() {
        let mut app = App::new(false, Arc::new(Mutex::new(Default::default())));
        app.modal.draft = "unsent".into();
        let mut buffer = HistoryBuffer::default();
        for (sequence, payload) in [
            (
                1,
                json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
            ),
            (
                2,
                json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":"OLD"}}),
            ),
            (
                3,
                json!({"type":"tool_execution_start","toolCallId":"old-tool","toolName":"read","args":{}}),
            ),
            (
                4,
                json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
            ),
            (
                5,
                json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":"NEW"}}),
            ),
            (
                6,
                json!({"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"NEW"}]}}),
            ),
        ] {
            buffer
                .push(pi_rpc::client::SequencedEvent {
                    sequence,
                    event: RpcEvent {
                        kind: payload["type"].as_str().unwrap().into(),
                        payload,
                    },
                })
                .unwrap();
        }
        buffer
            .apply_loaded(
                &mut app,
                &crate::session::Loaded {
                    id: "new-session".into(),
                    file: None,
                    data: json!({"entries":[],"leafId":null}),
                    boundary: 5,
                    discard_through: 3,
                    preserve_view: false,
                    notice: None,
                },
            )
            .unwrap();
        let snapshot = app.store.snapshot();
        assert_eq!(snapshot.messages.len(), 1);
        assert!(snapshot.tools.is_empty());
        assert_eq!(snapshot.messages[0]["content"][0]["text"], "NEW");
        assert_eq!(app.modal.draft, "unsent");
        assert_eq!(app.session, "new-session");
    }
}
