//! Real installed Pi tests: disposable config, seeded synthetic sessions, no provider.
use crate::{app::App, launch::Launch, session, shell::HistoryBuffer};
use pi_rpc::client::{Client, Limits};
use serde_json::json;
use std::{
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
#[tokio::test]
#[ignore = "requires explicit installed Pi; disposable sessions, no provider requests"]
async fn installed_pi_switch_veto_failure_and_restart() {
    let exe = std::env::var("PI_VISUAL_TEST_PI").expect("set installed Pi executable");
    let root = std::env::temp_dir().join(format!(
        "pi-visual-p041-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    for d in ["home", "agent", "project", "cache", "config"] {
        std::fs::create_dir(root.join(d)).unwrap();
    }
    let extension = root.join("transitions.ts");
    std::fs::write(
        &extension,
        include_str!("../../../tests/fixtures/session-transitions.ts"),
    )
    .unwrap();
    let a = root.join("a.jsonl");
    let b = root.join("b.jsonl");
    for (file, id, branched) in [(&a, "fixture-a", true), (&b, "fixture-b", false)] {
        let mut lines = vec![
            json!({"type":"session","version":3,"id":id,"timestamp":"2026-11-16T00:00:00Z","cwd":root.join("project")}),
        ];
        for (entry, parent, text) in if branched {
            vec![
                ("a0000001", None, "A root"),
                ("a0000002", Some("a0000001"), "A abandoned"),
                ("a0000003", Some("a0000001"), "A active"),
            ]
        } else {
            vec![("b0000001", None, "B root")]
        } {
            lines.push(json!({"type":"message","id":entry,"parentId":parent,"timestamp":"2026-11-16T00:00:00Z","message":{"role":"user","content":text,"timestamp":1794787200000u64}}));
        }
        std::fs::write(
            file,
            lines.iter().map(|v| format!("{v}\n")).collect::<String>(),
        )
        .unwrap();
    }
    let launch = Launch {
        executable: exe.into(),
        cwd: root.join("project"),
    };
    let command = |path: &str| {
        let mut c = launch.command(Some(path));
        c.env_clear()
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .env("HOME", root.join("home"))
            .env("PI_CODING_AGENT_DIR", root.join("agent"))
            .env("XDG_CACHE_HOME", root.join("cache"))
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("PI_OFFLINE", "1")
            .args([
                "--no-tools",
                "--no-extensions",
                "--no-mcp",
                "--no-skills",
                "--no-prompt-templates",
                "--no-themes",
                "--no-context-files",
                "-e",
            ])
            .arg(&extension);
        c
    };
    let mut client = Client::spawn_ordered(
        &mut command(a.to_str().unwrap()),
        Limits {
            event_capacity: 256,
            ..Default::default()
        },
    )
    .unwrap();
    let mut app = App::new(false, client.handle().ui());
    app.modal.draft = "unsent draft".into();
    async fn apply(client: &mut Client, app: &mut App, loaded: &session::Loaded) {
        let mut buffer = HistoryBuffer::default();
        while let Ok(e) = client.ordered_events.as_mut().unwrap().try_recv() {
            buffer.push(e).unwrap();
        }
        buffer.apply_loaded(app, loaded).unwrap();
    }
    let loaded = session::load(client.handle()).await.unwrap();
    apply(&mut client, &mut app, &loaded).await;
    assert_eq!(app.session, "fixture-a");
    assert_eq!(
        app.store
            .snapshot()
            .entries
            .values()
            .filter(|e| e["type"] == "message")
            .count(),
        3
    );
    let snapshot = app.store.snapshot();
    assert_eq!(
        snapshot
            .branch
            .iter()
            .filter(|id| snapshot.entries[*id]["type"] == "message")
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["a0000001", "a0000003"]
    );
    assert!(
        !app.transcript
            .entries
            .iter()
            .any(|e| e.raw.contains("abandoned"))
    );
    for target in [&b, &a, &b] {
        let loaded = session::transition(client.handle(), Some(target.to_str().unwrap().into()))
            .await
            .unwrap();
        assert!(loaded.discard_through > 0);
        apply(&mut client, &mut app, &loaded).await;
        assert_eq!(
            app.session,
            if target == &a {
                "fixture-a"
            } else {
                "fixture-b"
            }
        );
        assert_eq!(app.modal.draft, "unsent draft");
    }
    for target in [root.join("veto.jsonl"), root.join("project")] {
        let loaded = session::transition(client.handle(), Some(target.to_str().unwrap().into()))
            .await
            .unwrap();
        assert!(loaded.preserve_view);
        assert!(loaded.notice.is_some());
        apply(&mut client, &mut app, &loaded).await;
        assert_eq!(app.session, "fixture-b");
        assert_eq!(app.transcript.entries.len(), 1);
    }
    let old = client.pid();
    // Kill only this owned child; no unrelated process lookup or host shell.
    rustix::process::kill_process(
        rustix::process::Pid::from_raw(old.try_into().unwrap()).unwrap(),
        rustix::process::Signal::KILL,
    )
    .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !client.handle().is_closed() {
            client.health.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    app.disconnected();
    assert_eq!(app.modal.draft, "unsent draft");
    client
        .restart(&mut command(b.to_str().unwrap()))
        .await
        .unwrap();
    assert!(!PathBuf::from(format!("/proc/{old}")).exists());
    app.ui = client.handle().ui();
    let loaded = session::load(client.handle()).await.unwrap();
    apply(&mut client, &mut app, &loaded).await;
    assert_eq!(app.session, "fixture-b");
    assert_eq!(app.transcript.entries.len(), 1);
    assert_eq!(app.modal.draft, "unsent draft");
    let loaded = session::transition(client.handle(), None).await.unwrap();
    apply(&mut client, &mut app, &loaded).await;
    assert_ne!(app.session, "fixture-b");
    assert!(app.store.snapshot().messages.is_empty());
    let pid = client.pid();
    client.shutdown().await.unwrap();
    assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
    println!(
        "PASS real Pi A/B branches + veto + rejected switch + owned-child crash/restart + new session; no prompts/providers; {}",
        root.display()
    );
}
