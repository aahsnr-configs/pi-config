# P04.1 real Kitty acceptance — PENDING

2026-11-16,installed Kitty0.50.0. User reported `--demo transcript` and both normal live commands launched in
Kitty,with RPC ready,restored history and extmgr/pi-lens-lsp status delivery. The described cropped view proves
neither absence nor presence of header/input/footer, and proves no interaction.
No screenshot file, recording or developer-driven Kitty GUI test is claimed.
PTY/TestBackend evidence is listed separately in IMPLEMENTATION-STATUS.md.

No global kitty.conf, opacity or remote-control changes are required.

## Exact human steps

Build in this repository: `cargo build -p pi-visual --locked`.
In a real Kitty window run `./target/debug/pi-visual --demo transcript`.

| Check | Action and expected behavior | Real Kitty result |
|---|---|---|
| Launch/text | Offline and both live commands launch; ready/history/status reported | USER-REPORTED |
| ANSI fix | Rebuild/relaunch; extension statuses show plain text without [38;2;...m/[39m fragments; old values replaced | PENDING |
| Historical /quit | Restored /quit appears as history; frontend/backend remain running | PTY VERIFIED; Kitty follow-up PENDING |
| Regions | Top status; transcript; notice; bottom two-row input; cwd/cost/context footer | PENDING |
| Keyboard scroll | PgUp/PgDn; F6 then arrows reveal earlier/later cards | PENDING |
| Mouse wheel | Scroll older/newer content without sending or changing draft | PENDING |
| Latest | Ctrl+End returns to final aborted-run card | PENDING |
| Resize | Resize through40×12,60×18,80×24,100×30,120×40,160×50; regions remain usable | PENDING |
| Markdown/Unicode | Find code fence,中,é,🙂,table and link; no body/title loss at40+ columns | PENDING |
| Input | F6/click input; type long text; tail/insertion marker visible; grapheme Backspace; Ctrl+U clears | PENDING |
| Paste | Paste multiline Unicode; text stays local; Enter offline retains draft | PENDING |
| Stream anchor | PgUp, then several F2 presses; older reading position stays stable | PENDING |
| Raw details | Retry cards show structured data; F3 toggles safe original JSON | PENDING |
| Modal priority | Type draft,F4; wheel/shortcuts cannot approve; Escape cancels and restores draft/focus | PENDING |
| Exit | Ctrl+C outside modal; ordinary shell echo,cursor and selection work afterward | PENDING |
| Repeated lifecycle | Run/exit three times with no leftover capture/raw modes | PENDING |

For long history: `./target/debug/pi-visual --demo transcript-long`; repeat scroll,
latest and resize. This starts no Pi process or provider.

For genuine backend dialogs without live config/model calls:
`./target/debug/pi-visual --rpc-fixture /absolute/path/to/pi`.
Wait for ready,submit `/p03-dialogs`,then explicitly deny confirm with n,choose
second with arrows+Enter,type into input,submit editor with F2. Repeat/cancel with
Escape. Draft/focus and no accidental approval remain **PENDING** in real Kitty.

For controlled panic restoration in Kitty (test-only):

```bash
cargo test -p pi-visual terminal::tests::panic_restores_modes --locked -- --ignored --exact --nocapture
cargo test -p pi-visual shell::regression_tests::panic_with_child_and_reader_is_contained_and_joined --locked -- --ignored --exact --nocapture
```

Expected panic text is caught by the tests; both should finish successfully.
Check normal shell echo/cursor afterward. Real Kitty result: **PENDING**.

Normal startup smoke command from a separate coding project:
`/absolute/path/to/pi-visual --pi /absolute/path/to/pi --cwd /absolute/project`.
Wait for ready; do not submit a model prompt for a UI check. Authentication remains
Pi-owned. Isolated automated navigation/recovery is reproducible with
`python3 scripts/test-session-pty.py /absolute/path/to/pi`.

Record actual observer/date,Kitty version,dimensions,pass/failure and any capture
for each mandatory row. Until these interactions are observed,P04 stays PARTIAL.
P05 is not started automatically.

P04.2 automated additions: normal startup PTY now verifies header/input/footer,
keyboard focus,PageUp,wheel,Ctrl+End,ANSI status replacement and inert historical
/quit. These are not substitutes for genuine Kitty manual observations. Backend
ANSI colors/styles are intentionally stripped; semantic frontend colors remain.
