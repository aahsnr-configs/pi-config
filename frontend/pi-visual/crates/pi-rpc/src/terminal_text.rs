//! Display-only ECMA-48 control removal. Never forward untrusted escapes to a TTY.
//! Formatting is deliberately plain; Ratatui supplies the frontend's own styles.
//! Parse complete/reconstructed values independently: status updates replace,
//! rather than continue, previous strings. No carry state can hide another dialog.
#[derive(Clone, Copy)]
enum State {
    Text,
    Escape,
    Intermediates,
    Csi,
    String { osc: bool, escaped: bool },
}
fn text(c: char, out: &mut String) -> State {
    match c {
        '\x1b' => State::Escape,
        '\u{009b}' => State::Csi,
        '\u{009d}' => State::String {
            osc: true,
            escaped: false,
        },
        '\u{0090}' | '\u{0098}' | '\u{009e}' | '\u{009f}' => State::String {
            osc: false,
            escaped: false,
        },
        _ => {
            if (!c.is_control() || matches!(c, '\n' | '\t'))
                && !matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
            {
                out.push(c);
            }
            State::Text
        }
    }
}
/// Linear scan, constant parser state, no sequence buffer, output <= input bytes.
/// Truncated CSI/string sequences are discarded to end of this value. Malformed
/// CSI interrupted by a newline or non-ASCII text resumes safe text there.
/// A valid CSI final byte ends that sequence even if its command is unsupported.
pub(crate) fn plain(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut state = State::Text;
    for c in raw.chars() {
        state = match state {
            State::Text => text(c, &mut out),
            State::Escape => match c {
                '[' => State::Csi,
                ']' => State::String {
                    osc: true,
                    escaped: false,
                },
                'P' | 'X' | '^' | '_' => State::String {
                    osc: false,
                    escaped: false,
                },
                ' '..='/' => State::Intermediates,
                '0'..='~' => State::Text,
                _ => text(c, &mut out),
            },
            State::Intermediates => match c {
                ' '..='/' => State::Intermediates,
                '0'..='~' => State::Text,
                _ => text(c, &mut out),
            },
            State::Csi => match c {
                ' '..='?' => State::Csi,
                '@'..='~' => State::Text,
                _ => text(c, &mut out),
            },
            State::String { osc, escaped } => match c {
                '\u{0018}' | '\u{001a}' | '\u{009c}' => State::Text,
                '\x07' if osc => State::Text,
                '\\' if escaped => State::Text,
                _ => State::String {
                    osc,
                    escaped: c == '\x1b',
                },
            },
        };
    }
    out
}
