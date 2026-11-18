# Implementation status

**2026-11-16 — P04.2 ANSI rendering fix verified by automated/real-Pi/PTY tests.
P04 remains PARTIAL pending remaining genuine Kitty interaction checks. P05 not
started.** User reports both normal live startup commands reach RPC ready,restore
history and receive extmgr/pi-lens-lsp status. Post-fix Kitty appearance is PENDING.
Revised G1 remains PASS for permission interception only,not OS/network isolation.

## Root cause and trace

Installed pi-extmgr0.4.2 src/utils/status.ts:102 calls
`ctx.ui.setStatus("extmgr",ctx.ui.theme.fg("dim",...))`.
Installed pi-lens4.5.0 dist/index.js:159992–160009 similarly publishes theme-colored
pi-lens-lsp text. Source was inspected read-only; no live payload/config/session
dump,extension execution for discovery or installed-package modification.

Pi1.5.2 rpc-mode.js:105–113 serializes statusText unchanged. P02 JSONL/serde decode
and the stdout reader deliver that raw string to P03 UiState. setStatus uses a
BTreeMap insert by statusKey,so replacement semantics were already correct.
WorkflowStore does not append these values as messages; P04 reconciles keyed
view-state cards and a separate compact notice strip. Raw text survives there.

The defect was the shared reducer::display_text helper: it deleted ESC as a
control character but kept the rest of a sequence. Thus ESC[38;2;126;136;142m became
visible `[38;2;126;136;142m`,and reset ESC[39m became `[39m`. Both the Markdown/plain
layout path and direct Paragraph notice path used this helper. Ratatui never
interpreted those fragments; the frontend had already broken the ANSI sequences.

## Fix and guarantees

terminal_text.rs now consumes complete ESC/C1 CSI,OSC,DCS,SOS,PM,APC and ESC
intermediate sequences. All backend SGR colors/styles are deliberately stripped;
existing Ratatui semantic card/Markdown styles remain. This is the safe plain-text
fallback,not an ANSI terminal emulator or a P10 styling change. No dependency added.

The scanner is linear with constant parser state and output no larger than input.
Unsupported CSI commands are removed through their final byte; malformed CSI may
resume at safe newline/non-ASCII text; unterminated control strings/CSI are dropped
to this value's end. LF/tab and the existing bidi filter policy remain. No escape
or other terminal control is passed through for execution. Literal bracket text
without ESC/C1 remains literal; there is no heuristic deletion of source text.

Values are independent: a setStatus update REPLACES the previous value,not a stream
continuation. No parser state crosses status keys,records or permission dialogs.
Streamed message fragments are reconstructed raw first,then sanitized for display.
Original status strings/messages/options remain intact for diagnostics/replies.
Select labels that become blank or collide visually after sanitization fail closed;
valid explicit replies preserve the exact original option. No permission policy,
autoapproval or separate host execution route was added.

## Regression evidence

Baseline82 pass/6ignored. Four new display regressions failed before the fix and
then passed: truecolor/reset/unsupported CSI; OSC/DCS and C1/string controls;
truncated/malformed input; reconstructed fragments and idempotence. Additional
coverage proves colored/hidden selection labels cannot create ambiguous approval,
fragmented JSONL preserves raw status without reply,400 keyed updates retain only
two cards,removal affects only its key,streamed ANSI deltas render safely,and a
restored historical /quit remains data with draft/connection intact.

The normal real-Pi PTY now loads a reviewed synthetic ANSI-status fixture in a
private config root. It proves latest status replacement,no visible format
fragments,header/input/footer,keyboard focus,PageUp,wheel,Ctrl+End and inert
historical /quit; existing veto,restart,unsent draft and process/terminal cleanup
checks remain. The compact notice strip can repeat the current status summary;
that is separate from keyed transcript-card identity,not appended history.

| Exact command | Result |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo check --workspace --all-targets --locked | PASS |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS |
| cargo test --workspace --all-targets --locked | PASS:91 passed,0 failed,6 ignored |
| cargo build --workspace --locked | PASS |
| cargo build -p pi-visual --locked | PASS |
| cargo build --release --locked -p pi-visual | PASS |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture | PASS:1 real-Pi/local fake-provider test |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi PI_VISUAL_TEST_PACKAGES=/var/home/ahsan/.pi/agent/npm/node_modules cargo test -p pi-rpc --test g1 --locked -- --ignored --nocapture | PASS:1 integration,34 scenarios |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi cargo test -p pi-visual session_tests --locked -- --ignored --nocapture | PASS:real session lifecycle regression |
| python3 scripts/test-transcript-pty.py | PASS:offline replay,resize/input/modal and normal/panic restoration |
| python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi | PASS:preserved P03 dialogs |
| python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi --p04 | PASS:P04 real dialog integration |
| python3 scripts/test-session-pty.py /var/home/ahsan/.local/bin/pi | PASS:normal startup plus ANSI,/quit,chrome/focus/scroll and lifecycle checks |

Six default ignored tests remain G0,current G1,archived sandbox G1,real session
lifecycle and two PTY panic tests. All except the archived sandbox integration
were explicitly executed and passed. No previously passing test was removed,
disabled or weakened. No real provider credentials/paid calls were used.

## Review and changed files

Existing Astra Low advisor invoked once for a read-only P04.2 final trust-boundary
review. No material defect confirmed. Advisor inspected but did not independently
rerun tests; primary ran the commands above. Remaining evidence limits: real
Kitty post-fix interaction and broader Unicode visual-confusability defenses.

New: pi-rpc/src/terminal_text.rs,pi-rpc/tests/display.rs,
 tests/fixtures/ansi-status.ts and archived P04.1 status.
Changed: pi-rpc/src/{lib,reducer,extension_ui}.rs,pi-rpc/tests/extension_ui.rs,
 tui-shell/src/app.rs (tests only),scripts/test-session-pty.py,and relevant README/
 status/security/protocol/research/Kitty/interaction/acceptance docs.
No new dependency or Cargo.lock change. No Pi core,installed extension,live config,
credential,existing session or .codex workflow mutation. No Git/gh operation;
branch/SHA/tree cleanliness unverified under project rules.

## Acceptance and next step

Rebuild/relaunch with the same commands the user tested:
`cargo run -p pi-visual --locked -- --cwd /absolute/project`;
optionally `--pi /absolute/path/to/pi`. Offline demo remains
`cargo run -p pi-visual --locked -- --demo transcript`.

User-reported genuine Kitty evidence now includes both live launch commands,RPC
ready,restored conversation and extension status delivery. Post-fix status text,
actual Kitty footer/input/focus,keyboard/mouse/latest/resize,paste,modal cancel,
normal/panic recovery and repeated lifecycle remain PENDING in kitty-acceptance.md.
The actual installed extensions were not executed in developer tests; synthetic
payloads match their verified producer/wire formatting. No full extension or
Kitty acceptance is inferred from source inspection/PTYs. P04 remains PARTIAL.
No P05 graphics or P10 redesign was begun. Prior lifecycle/performance evidence:
docs/history/P04.1-IMPLEMENTATION-STATUS.md.
