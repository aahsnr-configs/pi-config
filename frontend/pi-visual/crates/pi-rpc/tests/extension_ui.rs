use pi_rpc::extension_ui::{Answer, Phase, UiState};
use serde_json::json;
use std::time::{Duration, Instant};
#[test]
fn typed_answers_and_exact_option_membership() {
    let now = Instant::now();
    let mut s = UiState::default();
    s.ingest(&json!({"type":"extension_ui_request","id":"a","method":"select","title":"Actual","options":["Allow","Block"]}),now).unwrap();
    assert!(s.queue("a", Answer::Value("invented".into()), now).is_err());
    let t = s.queue("a", Answer::Value("Block".into()), now).unwrap();
    assert_eq!(
        s.dispatch(t, now).unwrap(),
        json!({"type":"extension_ui_response","id":"a","value":"Block"})
    );
    assert!(s.queue("a", Answer::Cancel, now).is_err());
    assert_eq!(s.dialogs["a"].phase, Phase::Consumed);
}
#[test]
fn timeout_duplicate_reset_and_unknown_fail_closed() {
    let now = Instant::now();
    let mut s = UiState::default();
    let r = json!({"type":"extension_ui_request","id":"a","method":"confirm","title":"T","message":"M","timeout":1});
    s.ingest(&r, now).unwrap();
    let t = s.queue("a", Answer::Confirmed(true), now).unwrap();
    assert!(s.dispatch(t, now + Duration::from_millis(2)).is_err());
    assert!(s.ingest(&r, now).is_err());
    s.reset();
    assert!(s.queue("a", Answer::Confirmed(true), now).is_err());
    assert!(
        s.ingest(
            &json!({"type":"extension_ui_request","id":"u","method":"grant_all"}),
            now
        )
        .is_err()
    );
}
#[test]
fn fire_and_forget_never_creates_reply_and_keys_stay_distinct() {
    let mut s = UiState::default();
    for (id, k) in [("a", "ext-one"), ("b", "ext-two")] {
        s.ingest(&json!({"type":"extension_ui_request","id":id,"method":"setStatus","statusKey":k,"statusText":"working"}),Instant::now()).unwrap();
    }
    assert_eq!(s.status.len(), 2);
    assert!(s.dialogs.is_empty());
    assert!(s.queue("a", Answer::Cancel, Instant::now()).is_err());
}
#[tokio::test]
async fn one_way_reply_has_no_command_id_and_no_response_wait() {
    use pi_rpc::client::{Client, Limits};
    let mut cmd = tokio::process::Command::new("python3");
    cmd.env_clear().args(["-u","-c",r#"import sys,json
print(json.dumps(dict(type='extension_ui_request',id='backend-id',method='confirm',title='T',message='M')))
r=json.loads(sys.stdin.readline());assert r == dict(type='extension_ui_response',id='backend-id',confirmed=False)
print(json.dumps(dict(type='notify_seen')))
sys.stdin.read()"#]);
    let mut c = Client::spawn(&mut cmd, Limits::default()).unwrap();
    assert_eq!(c.events.recv().await.unwrap().kind, "extension_ui_request");
    c.handle()
        .respond_ui("backend-id", Answer::Confirmed(false))
        .await
        .unwrap();
    assert_eq!(c.events.recv().await.unwrap().kind, "notify_seen");
    assert_eq!(c.handle().pending_count(), 0);
    c.shutdown().await.unwrap();
}
#[test]
fn input_editor_confirm_and_cancel_shapes() {
    let now = Instant::now();
    for (method, answer, expected) in [
        (
            "input",
            Answer::Value("typed\n中".into()),
            json!({"type":"extension_ui_response","id":"id","value":"typed\n中"}),
        ),
        (
            "editor",
            Answer::Value("edited".into()),
            json!({"type":"extension_ui_response","id":"id","value":"edited"}),
        ),
        (
            "confirm",
            Answer::Confirmed(false),
            json!({"type":"extension_ui_response","id":"id","confirmed":false}),
        ),
        (
            "confirm",
            Answer::Cancel,
            json!({"type":"extension_ui_response","id":"id","cancelled":true}),
        ),
    ] {
        let mut s = UiState::default();
        s.ingest(&json!({"type":"extension_ui_request","id":"id","method":method,"title":"Actual title","message":"Actual message"}),now).unwrap();
        let t = s.queue("id", answer, now).unwrap();
        assert_eq!(s.dispatch(t, now).unwrap(), expected);
    }
}
#[test]
fn simultaneous_dialogs_generation_and_disconnect() {
    let now = Instant::now();
    let mut s = UiState::default();
    for id in ["a", "b"] {
        s.ingest(&json!({"type":"extension_ui_request","id":id,"method":"confirm","title":"t","message":"m"}),now).unwrap();
    }
    let a = s.queue("a", Answer::Confirmed(true), now).unwrap();
    let b = s.queue("b", Answer::Cancel, now).unwrap();
    assert_eq!(s.dispatch(b, now).unwrap()["id"], "b");
    s.cancel_all();
    assert!(s.dispatch(a, now).is_err());
    s.invalidate();
    assert!(s.queue("a", Answer::Confirmed(true), now).is_err());
}
#[test]
fn invalid_and_ambiguous_payloads_never_grant() {
    for v in [
        json!({"type":"extension_ui_request","id":"x","method":"select","title":"t","options":[true]}),
        json!({"type":"extension_ui_request","id":"x","method":"select","title":"t","options":["same","same\u{1b}"]}),
        json!({"type":"extension_ui_request","id":"x","method":"confirm","title":"t","message":1}),
        json!({"type":"extension_ui_request","id":"x","method":"input","title":"t","timeout":-1}),
    ] {
        let mut s = UiState::default();
        assert!(s.ingest(&v, Instant::now()).is_err());
        assert!(
            s.queue("x", Answer::Confirmed(true), Instant::now())
                .is_err()
        );
    }
}
#[test]
fn notifications_widgets_titles_and_text_are_bounded_data() {
    let mut s = UiState::default();
    let now = Instant::now();
    for v in [
        json!({"type":"extension_ui_request","id":"n","method":"notify","message":"\u{1b}]52;no\u{7}","notifyType":"warning"}),
        json!({"type":"extension_ui_request","id":"w","method":"setWidget","widgetKey":"ext","widgetLines":["raw"],"widgetPlacement":"belowEditor"}),
        json!({"type":"extension_ui_request","id":"t","method":"setTitle","title":"title"}),
        json!({"type":"extension_ui_request","id":"e","method":"set_editor_text","text":"backend"}),
        json!({"type":"extension_ui_request","id":"clear","method":"setWidget","widgetKey":"ext"}),
    ] {
        s.ingest(&v, now).unwrap();
    }
    assert_eq!(s.notifications.len(), 1);
    assert!(s.widgets.is_empty());
    assert_eq!(s.title.as_deref(), Some("title"));
    assert_eq!(s.editor_text.as_deref(), Some("backend"));
    assert!(s.dialogs.is_empty());
}
#[tokio::test]
async fn crashed_backend_invalidates_open_dialog() {
    use pi_rpc::client::{Client, Limits};
    let mut cmd = tokio::process::Command::new("python3");
    cmd.env_clear().args(["-u","-c","import json;print(json.dumps(dict(type='extension_ui_request',id='x',method='confirm',title='T',message='M')))"]);
    let mut c = Client::spawn(&mut cmd, Limits::default()).unwrap();
    c.events.recv().await.unwrap();
    while !c.health.borrow().closed {
        c.health.changed().await.unwrap();
    }
    assert!(
        c.handle()
            .respond_ui("x", Answer::Confirmed(true))
            .await
            .is_err()
    );
    assert_eq!(
        c.handle().ui().lock().unwrap().dialogs["x"].phase,
        Phase::Invalidated
    );
    c.shutdown().await.unwrap();
}
#[test]
fn routine_status_does_not_exhaust_dialog_replay_budget() {
    let mut s = UiState::default();
    for i in 0..1500 {
        s.ingest(&json!({"type":"extension_ui_request","id":format!("status-{i}"),"method":"setStatus","statusKey":"ext","statusText":"working"}),Instant::now()).unwrap();
    }
    s.ingest(&json!({"type":"extension_ui_request","id":"real-dialog","method":"confirm","title":"t","message":"m"}),Instant::now()).unwrap();
    assert!(
        s.queue("real-dialog", Answer::Confirmed(false), Instant::now())
            .is_ok()
    );
    assert_eq!(s.status.len(), 1);
}
#[tokio::test]
async fn cancelled_queued_approval_is_not_sent() {
    use pi_rpc::client::{Client, Limits};
    let mut cmd = tokio::process::Command::new("python3");
    cmd.env_clear().args([
        "-u",
        "-c",
        r#"import sys,json,time,select
print(json.dumps(dict(type='extension_ui_request',id='x',method='confirm',title='T',message='M')))
time.sleep(.2)
sys.stdin.readline()
time.sleep(.1)
if select.select([sys.stdin],[],[],.05)[0]:
 r=json.loads(sys.stdin.readline());assert r.get('type') != 'extension_ui_response'
print(json.dumps(dict(type='checked_no_reply')))
sys.stdin.read()"#,
    ]);
    let mut c = Client::spawn(&mut cmd, Limits::default()).unwrap();
    c.events.recv().await.unwrap();
    let h = c.handle();
    let writing = tokio::spawn({
        let h = h.clone();
        async move {
            h.request(
                json!({"type":"prompt","message":"x".repeat(900000)}),
                Duration::from_secs(2),
            )
            .await
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    let replying = tokio::spawn({
        let h = h.clone();
        async move { h.respond_ui("x", Answer::Confirmed(true)).await }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;
    replying.abort();
    let _ = replying.await;
    let event = tokio::time::timeout(Duration::from_secs(2), c.events.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.kind, "checked_no_reply");
    assert_eq!(
        h.ui().lock().unwrap().dialogs["x"].phase,
        Phase::Invalidated
    );
    c.shutdown().await.unwrap();
    let _ = writing.await;
}

#[test]
fn ansi_option_labels_must_be_visible_and_unambiguous_but_replies_stay_raw() {
    for options in [
        vec!["\x1b[31mAllow\x1b[0m", "\x1b[32mAllow\x1b[39m"],
        vec!["\x1b]52;c;hidden\x07", "Deny"],
        vec!["\x1b[38;2;", "Deny"],
    ] {
        let mut ui = UiState::default();
        assert!(ui.ingest(&json!({"type":"extension_ui_request","id":"permission","method":"select","title":"Permission","options":options}),Instant::now()).is_err());
        assert!(
            ui.queue(
                "permission",
                Answer::Value(options[0].into()),
                Instant::now()
            )
            .is_err()
        );
    }
    let mut ui = UiState::default();
    let raw = "\x1b[32mAllow\x1b[0m";
    ui.ingest(&json!({"type":"extension_ui_request","id":"valid","method":"select","title":"Permission","options":[raw,"Deny"]}),Instant::now()).unwrap();
    let ticket = ui
        .queue("valid", Answer::Value(raw.into()), Instant::now())
        .unwrap();
    assert_eq!(ui.dispatch(ticket, Instant::now()).unwrap()["value"], raw);
}

#[tokio::test]
async fn fragmented_jsonl_preserves_raw_status_and_never_creates_a_reply() {
    use pi_rpc::client::Client;
    use std::time::Duration;
    let mut command = tokio::process::Command::new("python3");
    command.env_clear().args(["-u","-c",r#"import sys,json,time
v=dict(type='extension_ui_request',id='status-id',method='setStatus',statusKey='extmgr',statusText='\x1b[38;2;126;136;142mDEMO ready\x1b[39m')
line=(json.dumps(v)+'\n').encode()
for i in range(0,len(line),3):
 sys.stdout.buffer.write(line[i:i+3]);sys.stdout.buffer.flush();time.sleep(.001)
sys.stdin.read()"#]);
    let mut client = Client::spawn(&mut command, Default::default()).unwrap();
    let event = tokio::time::timeout(Duration::from_secs(5), client.events.recv())
        .await
        .unwrap()
        .unwrap();
    let raw = "\x1b[38;2;126;136;142mDEMO ready\x1b[39m";
    assert_eq!(event.payload["statusText"], raw);
    assert_eq!(client.handle().ui().lock().unwrap().status["extmgr"], raw);
    assert_eq!(pi_rpc::reducer::display_text(raw), "DEMO ready");
    assert_eq!(client.handle().pending_count(), 0);
    assert!(client.handle().ui().lock().unwrap().dialogs.is_empty());
    client.shutdown().await.unwrap();
}
