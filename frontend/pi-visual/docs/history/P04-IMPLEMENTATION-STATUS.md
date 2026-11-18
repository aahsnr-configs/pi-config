# Implementation status

**2026-11-16 — P04 PARTIAL: automated + real PTY evidence passes; actual Kitty
manual acceptance NOT RUN.** Revised G1 remains PASS under permission-interception
scope (ADR0002); no OS isolation or production-readiness claim. P05 not started.

## Working slice

- Real Ratatui/Crossterm viewport: `cargo run -p pi-visual --locked -- --demo transcript`.
- Model-free replay of JSONL events and recorded get_state/get_entries responses;
 10,000-entry synthetic demo plus durable-history test; no backend for replay.
- Local Markdown headings, emphasis, links as safe text, fences/language labels,
 lists/tasks, quotes, rules and simple tables; grapheme/cell wrap and inert HTML.
- Observed-order message/tool/retry/aborted-run cards; independent concurrent tools;
 actual name/path/command/status/excerpt/details and supplied duration only.
- Height/revision/width cache, partial clipping, logical text anchoring, bottom
 follow/latest, bounded rendered cache and batch32 cold layout/reflow.
- Bounded terminal event loop, basic end-edit input/paste and row highlight;
 P03 modal priority, deadline/stale-ID validation and draft/focus preservation;
 keyed text status/widget/title/notification cards and explicit Ctrl+R staged editing.
- Isolated live UI fixture consumes the existing P02 client. Ordered history
 hydration preserves in-flight prefixes without duplicating saved messages.
- Cleanup owns/joins jobs, input reader and dedicated Pi child on normal errors,
 shutdown and controlled loop panic. No second process just for drawing.

## Files

New: tui-shell/src/{app,markdown,transcript,shell,terminal}.rs;
 tests/fixtures/p04-{transcript.jsonl,basic.golden}; scripts/test-transcript-pty.py;
 docs/interaction-spec.md and ADR0003.
Modified: tui-shell/{Cargo.toml,src/main.rs,src/modal.rs,src/dialog_demo.rs};
 pi-rpc/src/{client,reducer,extension_ui}.rs; pi-rpc/tests/reducer.rs;
 scripts/test-dialog-pty.py; Cargo.lock and affected README/architecture/protocol/
 security/dependency/research/parity/acceptance/status documentation.
Old public RPC APIs and default reducer limits preserved. Opt-in10k count retains
8MiB commit budget; oversized restore remains atomic. Cargo generated lockfile.
Git/gh NOT RUN under project rules: branch/SHA/cleanliness unverified; no commit,
remote or publish operation performed. Changes confined to workspace and disposable
/tmp fixtures; no Pi source/live settings/auth/session modification.

## Verification (fresh final run)

| Exact command | Result |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo check --workspace --all-targets | PASS |
| cargo clippy --workspace --all-targets -- -D warnings | PASS |
| cargo test --workspace --all-targets --locked | PASS:76 passed,0 failed,5 ignored |
| cargo build --workspace --locked | PASS |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture | PASS:1 real-Pi/local fake-provider test |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi PI_VISUAL_TEST_PACKAGES=/var/home/ahsan/.pi/agent/npm/node_modules cargo test -p pi-rpc --test g1 --locked -- --ignored --nocapture | PASS:1 real-Pi permission test,34 scenarios |
| python3 scripts/test-transcript-pty.py | PASS:offline viewport,mouse/resize/stream/modal,saved-history replay,FIFO/error rejection,Ctrl+C/SIGTERM and two panic lifecycle tests |
| python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi | PASS:existing P03 four-dialog keyboard flow |
| python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi --p04 | PASS:real Pi four-dialog flow in P04 viewport |
| cargo build --release --locked -p pi-visual | PASS |
| ./target/release/pi-visual --bench-transcript | PASS:measured below |

Five ignored defaults are G0, revised G1, archived sandbox G1 and two PTY panic
tests. All except the archived suite were explicitly executed by commands above.
Archived sandbox suite is compiled, not current acceptance. Default suite includes
P02/P03 regressions, six dimensions40×12/60×18/80×24/100×30/120×40/160×50,
Unicode/control/partial-fence cases, one reviewed text golden, cache eviction,
10k durable history, modal/paste/draft, delayed fenced hydration and200-record burst.

The PTY driver initially matched raw differential output and stale `input` text;
it failed although the screen had advanced. It now reconstructs displayed cells
before matching readiness/modal text. No production behavior or assertions were
weakened. A preservation regression also exposed missing P03 text-state presentation in
the new viewport: restored keyed text cards and staged editor Ctrl+R, without a
second policy engine or footer changes. A separate disconnect regression failed
before fixing App.connected;
transport already prevented sends, and both app/transport now reject submissions.

## Measured layout work (release, one local run)

`--bench-transcript`, fixed synthetic Markdown,100×25 viewport, reflow to80×25.
Times are CPU wall-clock durations of these functions; no terminal IO/frame/input
latency, jitter, RSS, battery or FPS claim. Cold/reflow numbers exclude event-loop
pacing delays. Warm column is100 calls to visible-row extraction, not100 frames.

| Entries | Cold layout μs |100 warm viewports μs|Reflow μs|Visible blocks|
|---:|---:|---:|---:|---:|
|100|380|55|383|4|
|1,000|4,277|55|4,409|4|
|10,000|215,485|57|219,811|4|

Separate P02 ingestion:200 events/100 messages3,069μs;
2,000 events/1,000 messages327,184μs. Reducer atomic clone/serialize and cold
measurement/reindexing remain history-size work; no O(visible) ingestion claim.
The8MiB accounted render cache excludes raw records, temporary allocations and
allocator overhead. Live reflow batches32 with event opportunities between batches.

## Astra architecture and final review

Initial: adopt one terminal/input owner; separate snapshots/ingestion from paint;
maintain observed event ordering, bounded layout/cache and logical anchors.
Final confirmed findings addressed: response/history race -> ordered local fence;
unrelated request failure releasing prompt guard -> typed job ownership;
FIFO blocking open -> O_NONBLOCK + opened-descriptor regular-file checks;
discarded paste -> bounded input/editor routing, no confirm/select answer;
evicted row anchor -> recover actual layout before recording logical offset;
panic-detached resources -> poll containment with owned awaitable cleanup.
Tests cover each. No third advisor call or claim of independent re-review of fixes.

## Remaining evidence and scope limits

P04 is PARTIAL under the requested strict acceptance criterion: real Kitty manual
interaction/capture is NOT RUN. Kitty0.50.0 is installed and TERM=xterm-kitty,
but these tool runs use pipes/PTYs rather than a verified desktop Kitty viewport.
Use README's exact Kitty commands/checklist before target acceptance. IME and
native terminal selection behavior are also NOT TESTED. No pixels/browser added.

Normal user-config startup is not wired; only offline and explicit isolated live
attachment are exposed. Hot reconnect/restart UI, session picker/tree navigation,
complete editor/copy/Vi/autocomplete, syntax lexing, aligned interactive tables,
reported cost/context and P10 styling remain outside this slice. Footer values
are unknown rather than guessed. Pure restore/session-switch logic is tested;
full live session-switch/restart user flows remain manual acceptance work.

Rendering/replay starts no model worker and adds no model-facing schema. Offline
UI has no Client at all; explicit connected prompt/abort/dialog/session commands
remain user actions. Full paired-provider G7 overhead accounting is NOT RUN.
G1 remains permission-only PASS; arbitrary approved code/MCP has host authority,
no four-mode/emergency architecture is claimed, and pi-sandbox is not required.

Next smallest slice: run the documented Kitty acceptance and isolated live session
switch/restore checks, resolve any defects and attach evidence. Do not begin P05
until separately requested. No browser, image compositor or component catalogue
was started.

---

# P03.1 baseline record (before P04)

**2026-11-16 — P03.1: revised G1 PASS (permission interception only).**
P02/G0 remains PASS. P03 is ready for acceptance under ADR 0002's approved scope.
Original sandbox-inclusive G1 remains historical PARTIAL. This record predates P04.
No production-ready, OS-isolation, universal path-denial or browser-security claim.

## Changes and preservation

- Production security requirement now names @gotgenes/pi-permission-system, or
  separately approved replacement; no pi-sandbox dependency or fork requirement.
- P02/P03 production Rust source, public APIs and UI behavior are unchanged.
- Previous real suite preserved as tests/g1_legacy_sandbox.rs (all 24 cases and
  assertions retained); opt-in archived evidence, not current acceptance.
- Current tests/g1.rs loads only permission-system plus real builtin MCP in MCP
  cases. No sandbox config or package is loaded by this suite.
- Extended security-provider.py for deterministic multi-call plans; added a real
  stdio MCP fixture permission-mcp.py. Both are test-only, not runtime tools.
- Original docs retained under docs/history/P03-original-*. Revised criteria and
  removed protections recorded in docs/ADRs/0002-permission-only-G1.md.
- Live Pi config/extensions/settings/auth/sessions untouched; no install/uninstall,
  Pi patch, new dependency, model request to a real provider, or Git/gh operation.
  Branch/SHA/tree cleanliness NOT CHECKED under no-agent-Git rules.

## Verification

Tested Pi1.5.2, @gotgenes/pi-permission-system42.0.0, Rust/Cargo1.100.0.

| Exact command | Outcome |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo check --workspace --all-targets | PASS |
| cargo clippy --workspace --all-targets -- -D warnings | PASS |
| cargo test --workspace --all-targets --locked | PASS: 57 passed, 0 failed, 3 opt-in ignored |
| cargo build --workspace --locked | PASS |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture | PASS: 1 real-Pi regression |
| PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi PI_VISUAL_TEST_PACKAGES=/var/home/ahsan/.pi/agent/npm/node_modules cargo test -p pi-rpc --test g1 --locked -- --ignored --nocapture | PASS: 1 integration containing 34 scenarios |
| python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi | PASS: real four-dialog terminal keyboard flow/draft restoration |

Ignored defaults are explicit G0, revised G1, and archived sandbox integration.
G0/G1 were run separately. Archived sandbox integration was compiled, not rerun:
it is no longer an acceptance dependency. Its prior passing evidence is retained.

34 scenarios cover allow/read/write/Bash; direct outside path/hard-deny/symlink/
traversal; ask denial/cancellation/timeout/once approval; session grant reuse plus
hard-deny and process-reset re-prompt; external read/write allow/deny and ask
approval/denial independently of path denial; Python/shell opaque scripts;
wrapper ask denial; sensitive simulated key read/write and Bash removal denial;
real MCP allow/deny/path-deny/ask and opaque-server effects. Denied operations
assert backend isError AND unchanged marker. MCP denial also asserts no server
execution receipt. No real credential files are used as sensitive-path fixtures.

Three adversarial approvals intentionally change a marker denied to direct
path-bearing tools: Python, shell script, opaque MCP tool. These demonstrate
lost containment; they are not recorded as prevented attacks. No automatic
unsandboxed fallback exists. The fixture's root-only guard protects the test
host and is not a production security mechanism.

## Review and corrections

Astra confirmed RPC single-select approval (native TUI double-confirmation is
not provided), interpreter/server limits, pre-execution-only symlink checks and
absence of a defined four-mode/emergency architecture. Its follow-up found
external_directory was always allow: added six external-gate cases with path
policy allowing so that independent read/write deny/ask decisions are proven.
All six now pass. No new production permission engine was introduced.

Initial MCP ask-deny fixture hit path deny before asking, so its required-dialog
assertion failed. Corrected that test's target to an allowed path to isolate MCP
ask; preserved the separate MCP path-deny case. All original assertions remain
in the historical suite. Revised integration then passed all 34 scenarios.

## Remaining limitations and acceptance boundary

Permission interception is not OS process/filesystem/network isolation. Trusted
extensions/MCP servers and approved arbitrary code can act outside inspectable
arguments. Static symlink resolution does not eliminate races. Backend dialog
abort/timeout has no immediate cancellation event; local deadline/generation/
settled/disconnect guards plus Pi's stale-ID handling remain the tested behavior.
Global status/widget keys have no origin identity. Draft-safe editor staging,
in-app title and notice-region widget presentation remain the P03 UI limits.

No approved four-mode implementation/emergency workflow was found; neither is
claimed tested. YOLO stayed false. If a separate approved four-mode specification
is supplied later it requires its own verification, not automatic acceptance.
Browser remains disabled; G2–G8, real Kitty/IME, remote/deferred/codemode MCP,
TOCTOU hardening, advisory/license audits and production resource QA remain untested.

P03 is ready for acceptance ONLY under revised permission-interception scope.
The subsequent P04 work was explicitly requested by the user. Do not reintroduce or recommend a pi-sandbox fork.
