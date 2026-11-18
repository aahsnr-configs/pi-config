use pi_rpc::{Incoming, ProtocolError, framing::JsonlReader};
use std::io::{BufReader, Cursor};

#[test]
fn reads_only_lf_boundaries_across_single_byte_chunks() {
    let bytes = "{\"type\":\"notify\",\"message\":\"中\u{2028}next\u{2029}🙂\"}\r\n{\"type\":\"agent_end\"}\n";
    let mut reader = JsonlReader::new(BufReader::with_capacity(1, Cursor::new(bytes)));
    let Incoming::Event(event) = reader.next_record().unwrap().unwrap() else {
        panic!()
    };
    assert_eq!(event.payload["message"], "中\u{2028}next\u{2029}🙂");
    assert!(matches!(
        reader.next_record().unwrap(),
        Some(Incoming::Event(_))
    ));
    assert!(reader.next_record().unwrap().is_none());
}

#[test]
fn oversized_record_is_drained_before_next_record() {
    let mut bytes = vec![b'x'; pi_rpc::MAX_RPC_LINE_BYTES + 10];
    bytes.extend_from_slice(b"\n{\"type\":\"agent_settled\"}\n");
    let mut reader = JsonlReader::new(BufReader::with_capacity(31, Cursor::new(bytes)));
    assert!(matches!(
        reader.next_record(),
        Err(pi_rpc::framing::ReadError::Protocol(
            ProtocolError::LineTooLong
        ))
    ));
    let Some(Incoming::Event(event)) = reader.next_record().unwrap() else {
        panic!()
    };
    assert_eq!(event.kind, "agent_settled");
}

#[test]
fn malformed_and_non_utf8_records_do_not_poison_following_records() {
    let mut reader = JsonlReader::new(Cursor::new(b"nope\n\xff\n{\"type\":\"ok\"}\n"));
    assert!(reader.next_record().is_err());
    assert!(matches!(
        reader.next_record(),
        Err(pi_rpc::framing::ReadError::InvalidUtf8)
    ));
    assert!(reader.next_record().unwrap().is_some());
}

#[test]
fn eof_never_commits_an_unterminated_record() {
    let mut reader = JsonlReader::new(Cursor::new(b"{\"type\":\"agent_settled\"}"));
    assert!(matches!(
        reader.next_record(),
        Err(pi_rpc::framing::ReadError::TruncatedRecord)
    ));
    assert!(reader.next_record().unwrap().is_none());
}

#[test]
fn accepts_exact_limit_with_optional_crlf() {
    let prefix = "{\"type\":\"event\",\"text\":\"";
    let line = format!(
        "{prefix}{}\"}}\r\n",
        "x".repeat(pi_rpc::MAX_RPC_LINE_BYTES - prefix.len() - 2)
    );
    let mut reader = JsonlReader::new(Cursor::new(line));
    assert!(reader.next_record().unwrap().is_some());
}
