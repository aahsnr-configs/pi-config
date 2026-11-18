//! Explicit disposable P03 modal demonstration, not normal Pi startup.
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use pi_rpc::client::{Client, Limits};
use std::{
    io,
    os::unix::fs::DirBuilderExt,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
pub fn run(executable: &str) -> io::Result<()> {
    if !Path::new(executable).is_absolute() {
        return Err(io::Error::other("Pi path must be absolute"));
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async_run(executable))
}
pub async fn fixture_client(executable: &str, ordered: bool) -> io::Result<Client> {
    if !Path::new(executable).is_absolute() {
        return Err(io::Error::other("Pi executable must be absolute"));
    }
    let root = std::env::temp_dir().join(format!(
        "pi-visual-ui-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    std::fs::DirBuilder::new().mode(0o700).create(&root)?;
    for d in ["home", "agent", "project", "cache", "config"] {
        std::fs::create_dir(root.join(d))?;
    }
    std::fs::write(
        root.join("ui.ts"),
        include_str!("../../../tests/fixtures/ui-dialogs.ts"),
    )?;
    let mut cmd = tokio::process::Command::new(executable);
    cmd.env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("PI_CODING_AGENT_DIR", root.join("agent"))
        .env("PI_OFFLINE", "1")
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .current_dir(root.join("project"))
        .args([
            "--no-session",
            "--no-tools",
            "--no-extensions",
            "--no-mcp",
            "--no-skills",
            "--no-prompt-templates",
            "--no-themes",
            "--no-context-files",
            "-e",
        ])
        .arg(root.join("ui.ts"));
    let client = if ordered {
        Client::spawn_ordered
    } else {
        Client::spawn
    }(
        &mut cmd,
        Limits {
            event_capacity: 256,
            ..Limits::default()
        },
    )
    .map_err(io::Error::other)?;
    Ok(client)
}
async fn async_run(executable: &str) -> io::Result<()> {
    let mut client = fixture_client(executable, false).await?;
    let h = client.handle();
    let state = h.ui();
    let mut modal = crate::modal::Modal::default();
    modal.draft = "Preserved local draft (demo data)".into();
    let mut tasks = tokio::task::JoinSet::new();
    let trigger = h.clone();
    tasks.spawn(async move {
        trigger
            .request(
                serde_json::json!({"type":"prompt","message":"/p03-dialogs"}),
                Duration::from_secs(3600),
            )
            .await
            .map(|_| ())
    });
    let mut terminal = ratatui::init();
    let result: io::Result<()> = async {
        loop {
            while client.events.try_recv().is_ok() {}
            if client.health.borrow().closed || client.health.borrow().desynchronized {
                state.lock().unwrap().invalidate();
                return Err(io::Error::other(
                    "RPC disconnected or invalid UI request; dialogs invalidated",
                ));
            }
            modal.sync(&state);
            terminal.draw(|f| modal.draw(f, &state))?;
            if event::poll(Duration::ZERO)?
                && let Event::Key(key) = event::read()?
            {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    state.lock().unwrap().invalidate();
                    break;
                }
                if modal.active.is_none()
                    && key.code == KeyCode::Char('r')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    if let Some(text) = state.lock().unwrap().editor_text.take() {
                        modal.draft = text;
                    }
                    continue;
                }
                if key.code == KeyCode::Char('g') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    state.lock().unwrap().cancel_all();
                    let handle = h.clone();
                    tasks.spawn(async move {
                        handle
                            .request(serde_json::json!({"type":"abort"}), Duration::from_secs(5))
                            .await
                            .map(|_| ())
                    });
                    continue;
                }
                if let Some((id, answer)) = modal.key(key) {
                    let handle = h.clone();
                    tasks.spawn(async move { handle.respond_ui(&id, answer).await });
                }
            }
            while let Some(result) = tasks.try_join_next() {
                result
                    .map_err(io::Error::other)?
                    .map_err(io::Error::other)?;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(())
    }
    .await;
    ratatui::restore();
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    let close = client.shutdown().await.map_err(io::Error::other);
    result.and(close)
}
