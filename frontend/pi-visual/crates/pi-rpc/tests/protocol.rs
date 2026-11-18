use pi_rpc::{Incoming, ProtocolError, RequestTracker, encode_line, parse_line, read_only_request};
use serde_json::json;

#[test]
fn read_only_command_is_valid_jsonl() {
    let command = read_only_request("probe-1", "get_state").expect("read-only command");
    assert_eq!(command, json!({"id":"probe-1","type":"get_state"}));
    let serialized = encode_line(&command).expect("serialize");
    assert_eq!(serialized, "{\"id\":\"probe-1\",\"type\":\"get_state\"}\n");
}

#[test]
fn correlates_out_of_order_responses_amid_events() {
    let mut tracker = RequestTracker::default();
    tracker.register("one").expect("register one");
    tracker.register("two").expect("register two");
    assert_eq!(
        tracker.register("one"),
        Err(ProtocolError::DuplicateRequestId)
    );
    let frames = [
        r#"{"type":"agent_start"}"#,
        r#"{"type":"response","id":"two","command":"get_commands","success":true,"data":{"commands":[]}}"#,
        r#"{"type":"response","id":"one","command":"get_state","success":true,"data":{}}"#,
        r#"{"type":"agent_settled","aborted":false}"#,
    ];
    let mut events = 0;
    for frame in frames {
        match parse_line(frame).expect("frame") {
            Incoming::Event(_) => events += 1,
            Incoming::Response(response) => tracker.complete(&response).expect("matched"),
        }
    }
    assert_eq!(events, 2);
    assert!(tracker.is_empty());
}

#[test]
fn rejects_malformed_response_but_preserves_extension_ui_event() {
    assert_eq!(
        parse_line(r#"{"type":"response","command":"get_state","success":"yes"}"#),
        Err(ProtocolError::InvalidEnvelope("success"))
    );
    let event = parse_line(
        r#"{"type":"extension_ui_request","id":"dlg-1","method":"confirm","message":"Allow?"}"#,
    )
    .expect("extension UI event");
    let Incoming::Event(event) = event else {
        panic!("expected UI event");
    };
    assert_eq!(event.kind, "extension_ui_request");
    assert_eq!(event.payload["id"], "dlg-1");
}

#[test]
fn rejects_oversized_jsonl() {
    let line = format!(
        "{{\"type\":\"tool_execution_update\",\"data\":\"{}\"}}",
        "x".repeat(8 * 1024 * 1024)
    );
    assert_eq!(parse_line(&line), Err(ProtocolError::LineTooLong));
}
#[test]
fn response_extensions_survive_and_invalid_error_is_rejected() {
    let Incoming::Response(r) = parse_line(
        r#"{"type":"response","id":"x","command":"get_state","success":true,"future":17}"#,
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(r.extra["future"], 17);
    assert_eq!(
        parse_line(r#"{"type":"response","command":"get_state","success":false,"error":123}"#),
        Err(ProtocolError::InvalidEnvelope("error"))
    );
}
