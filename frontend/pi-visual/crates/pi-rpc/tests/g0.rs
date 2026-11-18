//! Opt-in real installed Pi + local fake provider. No live config or paid requests.
use pi_rpc::{
    client::{Client, Limits},
    reducer::WorkflowStore,
};
use serde_json::json;
use std::{
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
#[tokio::test]
#[ignore = "requires installed Pi; loopback-only fake provider, explicit opt-in"]
async fn installed_pi_fake_provider() {
    let executable = std::env::var("PI_VISUAL_TEST_PI").expect("set explicit installed Pi path");
    let root = std::env::temp_dir().join(format!(
        "pi-visual-g0-{}-{}",
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
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/fake-provider.py");
    let mut provider = Command::new("python3")
        .env_clear()
        .args(["-u", script.to_str().unwrap()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut port = String::new();
    tokio::time::timeout(
        Duration::from_secs(3),
        BufReader::new(provider.stdout.take().unwrap()).read_line(&mut port),
    )
    .await
    .unwrap()
    .unwrap();
    let port: u16 = port
        .trim()
        .parse()
        .expect("loopback fixture did not start; local networking must be permitted");
    let model = json!({"providers":{"fixture":{"baseUrl":format!("http://127.0.0.1:{}/v1",port),"api":"openai-completions","apiKey":"local-test-only","models":[{"id":"fixture","reasoning":false,"input":["text"],"contextWindow":32768,"maxTokens":128,"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0}}]}}});
    std::fs::write(root.join("agent/models.json"), model.to_string()).unwrap();
    let mut command = Command::new(executable);
    command
        .env_clear()
        .env("PI_OFFLINE", "1")
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("PI_CODING_AGENT_DIR", root.join("agent"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("XDG_CONFIG_HOME", root.join("config"))
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
            "--provider",
            "fixture",
            "--model",
            "fixture",
        ]);
    let mut client = Client::spawn(&mut command, Limits::default()).unwrap();
    let h = client.handle();
    assert!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(10))
            .await
            .unwrap()
            .success
    );
    assert!(
        h.request(
            json!({"type":"prompt","message":"Return the fixture text"}),
            Duration::from_secs(10)
        )
        .await
        .unwrap()
        .success
    );
    let mut store = WorkflowStore::default();
    let mut trace = String::new();
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(event) = client.events.recv().await {
            trace.push_str(&event.payload.to_string());
            trace.push('\n');
            store.apply(&event).unwrap();
            if store.snapshot().settled {
                break;
            }
        }
    })
    .await
    .unwrap();
    assert!(store.snapshot().settled);
    std::fs::write(root.join("rpc-fixture-events.jsonl"), trace).unwrap();
    assert!(store.snapshot().messages.iter().any(|m| {
        m["role"] == "assistant"
            && m["content"]
                .as_array()
                .is_some_and(|a| a.iter().any(|b| b["text"] == "local fixture"))
    }));
    let entries = h
        .request(json!({"type":"get_entries"}), Duration::from_secs(2))
        .await
        .unwrap();
    store
        .restore("isolated", entries.data.as_ref().unwrap(), false)
        .unwrap();
    assert!(!store.snapshot().branch.is_empty());
    client.shutdown().await.unwrap();
    assert!(h.is_closed());
    assert!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(1))
            .await
            .is_err()
    );
    provider.kill().await.unwrap();
    let _ = provider.wait().await;
    println!(
        "G0 real Pi state + fake-provider stream + entries + shutdown PASS; disposable root {}",
        root.display()
    );
}
