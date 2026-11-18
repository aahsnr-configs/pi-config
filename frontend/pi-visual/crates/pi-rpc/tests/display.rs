//! Synthetic untrusted terminal text; originals must remain outside display output.
use pi_rpc::reducer::display_text;
#[test]
fn truecolor_resets_and_unsupported_csi_leave_no_fragments() {
    for raw in [
        "\x1b[38;2;126;136;142mReady\x1b[39m",
        "\x1b[1;4;48;2;1;2;3mReady\x1b[0m",
        "\x1b[2J\x1b[H\x1b[?25lReady\x1b[?25h",
        "\u{009b}38;2;126;136;142mReady\u{009b}0m",
    ] {
        assert_eq!(display_text(raw), "Ready");
    }
    assert_eq!(
        display_text("literal [39m and [1,2]"),
        "literal [39m and [1,2]"
    );
}
#[test]
fn osc_dcs_and_other_control_strings_cannot_leak_or_execute() {
    for sequence in [
        "\x1b]52;c;clipboard-secret\x07",
        "\x1b]8;;https://example.invalid\x1b\\",
        "\x1bPpayload\x1b[31mhidden\x1b\\",
        "\x1b_hidden\x1b\\",
        "\x1b^hidden\x1b\\",
        "\x1bXhidden\x1b\\",
        "\u{009d}52;c;clipboard-secret\u{009c}",
        "\u{0090}hidden\u{009c}",
    ] {
        assert_eq!(
            display_text(&format!("before{sequence}after")),
            "beforeafter"
        );
    }
    assert_eq!(
        display_text("a\x1b(0b\x1b(Bc\r\x08\x7f\u{202e}\n\t中"),
        "abc\n\t中"
    );
}
#[test]
fn truncated_and_malformed_sequences_are_bounded_and_do_not_cross_values() {
    for suffix in [
        "\x1b",
        "\x1b[38;2;",
        "\x1b]52;unterminated",
        "\x1bPunterminated",
        "\x1b(",
    ] {
        assert_eq!(display_text(&format!("safe{suffix}")), "safe");
    }
    assert_eq!(display_text("\x1b[38;2;\nNext"), "\nNext");
    assert_eq!(display_text("\x1b[38;2;🙂Next"), "🙂Next");
    assert_eq!(
        display_text("next independent status"),
        "next independent status"
    );
    let huge = format!("safe\x1b]{}", "x".repeat(100000));
    assert_eq!(display_text(&huge), "safe");
}
#[test]
fn reconstructed_fragments_and_repeated_sanitization_are_safe() {
    let mut assembled = String::new();
    for part in ["\x1b[38;2;", "126;136;142", "mReady", "\x1b[", "39m"] {
        assembled.push_str(part);
        let shown = display_text(&assembled);
        assert!(!shown.contains('['));
        assert!(!shown.contains('\x1b'));
        assert_eq!(display_text(&shown), shown);
    }
    assert_eq!(display_text(&assembled), "Ready");
    assert!(assembled.contains("\x1b[38;2;126;136;142m"));
}
