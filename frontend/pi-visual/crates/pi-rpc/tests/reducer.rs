use pi_rpc::{Incoming, parse_line, reducer::WorkflowStore};
use serde_json::json;
fn apply(s: &mut WorkflowStore, v: serde_json::Value) {
    let Incoming::Event(e) = parse_line(&v.to_string()).unwrap() else {
        panic!()
    };
    s.apply(&e).unwrap();
}
#[test]
fn deltas_and_authoritative_finals_do_not_double_append() {
    let mut s = WorkflowStore::default();
    apply(
        &mut s,
        json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
    );
    for t in ["ha", "ha"] {
        apply(
            &mut s,
            json!({"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":t}}),
        );
    }
    assert_eq!(s.snapshot().messages[0]["content"][0]["text"], "haha");
    apply(
        &mut s,
        json!({"type":"message_update","assistantMessageEvent":{"type":"text_end","contentIndex":0,"content":"final"}}),
    );
    assert_eq!(s.snapshot().messages[0]["content"][0]["text"], "final");
    let end = json!({"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"authoritative"}]}});
    apply(&mut s, end.clone());
    apply(&mut s, end);
    assert_eq!(s.snapshot().messages.len(), 1);
}
#[test]
fn concurrent_tools_retry_and_settled_are_distinct() {
    let mut s = WorkflowStore::default();
    apply(&mut s, json!({"type":"agent_start"}));
    for id in ["a", "b"] {
        apply(
            &mut s,
            json!({"type":"tool_execution_start","toolCallId":id,"toolName":"read","args":{}}),
        );
    }
    apply(
        &mut s,
        json!({"type":"tool_execution_end","toolCallId":"a","isError":true,"result":{}}),
    );
    apply(
        &mut s,
        json!({"type":"agent_end","willRetry":true,"messages":[]}),
    );
    assert!(!s.snapshot().settled);
    assert_eq!(s.snapshot().tools.len(), 2);
    apply(&mut s, json!({"type":"auto_retry_start","attempt":1}));
    apply(&mut s, json!({"type":"agent_settled","aborted":true}));
    assert!(s.snapshot().settled);
    assert_eq!(s.snapshot().aborted, Some(true));
}
#[test]
fn history_keeps_branches_and_leaf_can_move_without_new_entries() {
    let mut s = WorkflowStore::default();
    s.restore("s",&json!({"entries":[{"id":"a","parentId":null,"type":"message","message":{"role":"user","content":"root"}},{"id":"b","parentId":"a","type":"message","message":{"role":"assistant","content":[]}},{"id":"c","parentId":"a","type":"message","message":{"role":"assistant","content":[]}}],"leafId":"b"}),false).unwrap();
    s.restore("s", &json!({"entries":[],"leafId":"c"}), true)
        .unwrap();
    assert_eq!(s.snapshot().branch, vec!["a", "c"]);
    assert_eq!(s.snapshot().entries.len(), 3);
    let before = s.snapshot();
    assert!(
        s.restore("s", &json!({"entries":[],"leafId":"missing"}), true)
            .is_err()
    );
    assert_eq!(s.snapshot(), before);
}
#[test]
fn invalid_index_is_atomic() {
    let mut s = WorkflowStore::default();
    apply(
        &mut s,
        json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
    );
    let before = s.snapshot();
    let Incoming::Event(e)=parse_line(r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":999999,"delta":"x"}}"#).unwrap() else {panic!()};
    assert!(s.apply(&e).is_err());
    assert_eq!(s.snapshot(), before);
}
#[test]
fn thinking_tool_arguments_and_tree_replay() {
    let mut s = WorkflowStore::default();
    apply(
        &mut s,
        json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
    );
    for u in [
        json!({"type":"thinking_delta","contentIndex":0,"delta":"hmm"}),
        json!({"type":"thinking_end","contentIndex":0,"content":"done"}),
        json!({"type":"toolcall_start","contentIndex":1,"id":"t","toolName":"read"}),
        json!({"type":"toolcall_delta","contentIndex":1,"delta":"{\"path\":"}),
        json!({"type":"toolcall_end","contentIndex":1,"toolCall":{"type":"toolCall","id":"t","name":"read","arguments":{"path":"a"}}}),
    ] {
        apply(
            &mut s,
            json!({"type":"message_update","assistantMessageEvent":u}),
        );
    }
    assert_eq!(s.snapshot().messages[0]["content"][0]["thinking"], "done");
    assert_eq!(
        s.snapshot().messages[0]["content"][1]["arguments"]["path"],
        "a"
    );
    s.restore_tree("new",&json!({"tree":[{"entry":{"id":"r","parentId":null,"type":"message","message":{"role":"user","content":"saved"}},"children":[]}],"leafId":"r"})).unwrap();
    assert_eq!(s.snapshot().messages[0]["content"], "saved");
    assert!(
        s.restore("old", &json!({"entries":[],"leafId":"r"}), true)
            .is_err()
    );
}
#[test]
fn raw_text_is_preserved_but_display_controls_are_neutralized() {
    let raw = "x\u{1b}]52;bad\u{7}\u{202e}y";
    let display = pi_rpc::reducer::display_text(raw);
    assert!(!display.contains('\u{1b}'));
    assert!(!display.contains('\u{202e}'));
    assert!(raw.contains('\u{1b}'));
}
#[test]
fn unknown_compatible_update_does_not_require_content_index() {
    let mut s = WorkflowStore::default();
    apply(
        &mut s,
        json!({"type":"message_start","message":{"role":"assistant","content":[]}}),
    );
    apply(
        &mut s,
        json!({"type":"message_update","assistantMessageEvent":{"type":"future_metadata","value":1}}),
    );
    assert!(
        s.snapshot().messages[0]["content"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn representative_synthetic_wire_fixture_replays() {
    let mut s = WorkflowStore::default();
    for line in include_str!("../../../tests/fixtures/rpc-stream-p02.jsonl").lines() {
        let Incoming::Event(e) = parse_line(line).unwrap() else {
            panic!()
        };
        s.apply(&e).unwrap();
    }
    assert!(s.snapshot().settled);
    assert_eq!(
        s.snapshot().messages[0]["content"][1]["text"],
        "hello fixture"
    );
}
#[test]
fn explicit_frontend_capacity_preserves_default_and_byte_budget() {
    let entries:Vec<_>=(0..10000).map(|i|json!({"type":"message","id":format!("e{i}"),"parentId":if i==0{None}else{Some(format!("e{}",i-1))},"message":{"role":"user","content":"fixture"}})).collect();
    let data = json!({"entries":entries,"leafId":"e9999"});
    assert!(WorkflowStore::default().restore("s", &data, false).is_err());
    let mut s = WorkflowStore::with_entry_capacity(10000).unwrap();
    s.restore("s", &data, false).unwrap();
    assert_eq!(s.snapshot().messages.len(), 10000);
    assert!(WorkflowStore::with_entry_capacity(10001).is_err());
    let before = s.snapshot();
    let oversized = json!({"entries":[{"type":"message","id":"large","parentId":null,"message":{"role":"user","content":"x".repeat(8 * 1024 * 1024)}}],"leafId":"large"});
    assert!(s.restore("large", &oversized, false).is_err());
    assert_eq!(s.snapshot(), before);
}
