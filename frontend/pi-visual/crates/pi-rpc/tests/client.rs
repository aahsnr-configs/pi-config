use pi_rpc::client::{Client, Limits};
use serde_json::json;
use std::time::Duration;
use tokio::process::Command;
fn fake(code: &str) -> Command {
    let mut c = Command::new("python3");
    c.env_clear().args(["-u", "-c", code]);
    c
}
#[tokio::test]
async fn concurrent_correlation_and_events() {
    let mut c = fake(
        "import sys,json\na=json.loads(sys.stdin.readline()); b=json.loads(sys.stdin.readline())\nprint('{\"type\":\"agent_start\"}')\nfor r in [b,a]: print(json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True,data=r['type'])))\nsys.stdin.read()",
    );
    let mut s = Client::spawn(&mut c, Limits::default()).unwrap();
    let h = s.handle();
    let (a, b) = tokio::join!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(2)),
        h.request(json!({"type":"get_commands"}), Duration::from_secs(2))
    );
    assert_eq!(a.unwrap().command, "get_state");
    assert_eq!(b.unwrap().command, "get_commands");
    assert_eq!(s.events.recv().await.unwrap().kind, "agent_start");
    assert_eq!(h.pending_count(), 0);
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn cancellation_timeout_restart_and_crash() {
    let mut c = fake("import sys,time\nfor l in sys.stdin: time.sleep(60)");
    let mut s = Client::spawn(&mut c, Limits::default()).unwrap();
    let h = s.handle();
    assert!(
        h.request(json!({"type":"get_state"}), Duration::from_millis(30))
            .await
            .is_err()
    );
    assert_eq!(h.pending_count(), 0);
    let task = tokio::spawn({
        let h = h.clone();
        async move {
            h.request(json!({"type":"get_state"}), Duration::from_secs(60))
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    task.abort();
    let _ = task.await;
    assert_eq!(h.pending_count(), 0);
    let mut crash = fake("import sys; sys.exit(7)");
    s.restart(&mut crash).await.unwrap();
    assert!(
        s.handle()
            .request(json!({"type":"get_state"}), Duration::from_secs(1))
            .await
            .is_err()
    );
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn overflow_is_terminal_and_stderr_is_content_free() {
    let mut c = fake(
        "import sys\nsys.stderr.write('secret\\x1b]52;bad\\x07' * 10000); sys.stderr.flush()\nfor i in range(100): print('{\"type\":\"notify\"}')\nsys.stdin.read()",
    );
    let s = Client::spawn(
        &mut c,
        Limits {
            event_capacity: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let h = s.handle();
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(h.is_closed());
    assert_eq!(h.pending_count(), 0);
    assert!(h.diagnostic_bytes() > 0);
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn fragmented_bad_oversized_records_recover_without_desync() {
    let mut c = fake(
        "import sys,json,time\nr=json.loads(sys.stdin.readline())\nsys.stdout.write('bad\\n'+'x'*200+'\\n');sys.stdout.flush()\ns=json.dumps(dict(type='response',id=r['id'],command=r['type'],success=True))+'\\n'\nfor b in s: sys.stdout.write(b);sys.stdout.flush()\nsys.stdin.read()",
    );
    let s = Client::spawn(
        &mut c,
        Limits {
            record_bytes: 128,
            ..Limits::default()
        },
    )
    .unwrap();
    let h = s.handle();
    assert!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(2))
            .await
            .unwrap()
            .success
    );
    assert_eq!(h.rejected_records(), 2);
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn blocked_writer_does_not_prevent_shutdown() {
    let mut c = fake("import time;time.sleep(60)");
    let s = Client::spawn(&mut c, Limits::default()).unwrap();
    let h = s.handle();
    let task = tokio::spawn(async move {
        h.request(
            json!({"type":"prompt","message":"x".repeat(900000)}),
            Duration::from_secs(10),
        )
        .await
    });
    tokio::time::sleep(Duration::from_millis(30)).await;
    tokio::time::timeout(Duration::from_secs(2), s.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert!(task.await.unwrap().is_err());
}
#[tokio::test]
async fn mismatched_reply_is_protocol_error_and_old_handle_stays_closed() {
    let mut c = fake(
        "import sys,json\nr=json.loads(sys.stdin.readline())\nprint(json.dumps(dict(type='response',id=r['id'],command='prompt',success=True)))\nsys.stdin.read()",
    );
    let mut s = Client::spawn(&mut c, Limits::default()).unwrap();
    let h = s.handle();
    assert_eq!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(2))
            .await
            .unwrap_err(),
        pi_rpc::client::ClientError::Protocol
    );
    let mut c = fake("import sys;sys.stdin.read()");
    s.restart(&mut c).await.unwrap();
    assert!(h.is_closed());
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn late_response_does_not_match_next_request() {
    let mut c = fake(
        "import sys,json,time\na=json.loads(sys.stdin.readline());time.sleep(.08)\nprint(json.dumps(dict(type='response',id=a['id'],command=a['type'],success=True)))\nb=json.loads(sys.stdin.readline());print(json.dumps(dict(type='response',id=b['id'],command=b['type'],success=True)))\nsys.stdin.read()",
    );
    let s = Client::spawn(&mut c, Limits::default()).unwrap();
    let h = s.handle();
    assert_eq!(
        h.request(json!({"type":"get_state"}), Duration::from_millis(20))
            .await
            .unwrap_err(),
        pi_rpc::client::ClientError::Timeout
    );
    assert_eq!(
        h.request(json!({"type":"get_commands"}), Duration::from_secs(2))
            .await
            .unwrap()
            .command,
        "get_commands"
    );
    assert_eq!(h.pending_count(), 0);
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn dropping_session_closes_handles_and_bounds_pending() {
    let mut c = fake("import sys,time;sys.stdin.readline();time.sleep(60)");
    let s = Client::spawn(
        &mut c,
        Limits {
            pending_requests: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let h = s.handle();
    let t = tokio::spawn({
        let h = h.clone();
        async move {
            h.request(json!({"type":"get_state"}), Duration::from_secs(10))
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(
        h.request(json!({"type":"get_state"}), Duration::from_secs(1))
            .await
            .unwrap_err(),
        pi_rpc::client::ClientError::Capacity
    );
    drop(s);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), t)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert!(h.is_closed());
    assert_eq!(h.pending_count(), 0);
}
#[tokio::test]
async fn cancelled_restart_retains_reaper_and_inherited_pipes_are_cleaned() {
    use std::{future::Future, task::Poll};
    let mut c = fake("import os,time;os.fork();time.sleep(60)");
    let mut s = Client::spawn(&mut c, Limits::default()).unwrap();
    let old = s.pid();
    let mut replacement = fake("import sys;sys.stdin.read()");
    {
        let mut restart = std::pin::pin!(s.restart(&mut replacement));
        std::future::poll_fn(|cx| {
            assert!(restart.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
    }
    s.restart(&mut replacement).await.unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{old}")).exists());
    let new = s.pid();
    s.shutdown().await.unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{new}")).exists());
}
#[tokio::test]
async fn malformed_stream_notifies_consumer_to_reload_history() {
    let mut c = fake(
        "import sys,time\nprint('{\"type\":\"agent_start\"}')\nprint('bad json')\nprint('{\"type\":\"agent_settled\",\"aborted\":false}')\nsys.stdin.read()",
    );
    let mut s = Client::spawn(&mut c, Limits::default()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if s.health.borrow().desynchronized {
                break;
            }
            s.health.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(s.handle().rejected_records(), 1);
    s.shutdown().await.unwrap();
}
#[tokio::test]
async fn crash_and_drop_reap_direct_child() {
    let mut c = fake("import sys;sys.exit(7)");
    let s = Client::spawn(&mut c, Limits::default()).unwrap();
    let pid = s.pid();
    s.shutdown().await.unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    let mut c = fake("import time;time.sleep(60)");
    let s = Client::spawn(&mut c, Limits::default()).unwrap();
    let pid = s.pid();
    drop(s);
    tokio::time::timeout(Duration::from_secs(2), async {
        while std::path::Path::new(&format!("/proc/{pid}")).exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}
