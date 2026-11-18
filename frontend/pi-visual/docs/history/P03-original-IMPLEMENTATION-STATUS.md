> HISTORICAL P03 record. Superseded by approved P03.1 scope; not current requirements or recommendations.

# Implementation status

**2026-11-16: P02/G0 PASS; P03/G1 PARTIAL. P03 is NOT COMPLETE.**
P04 has not begun. Normal live chat, widgets and browser rendering are absent.

## Preserved P02 foundation

Protocol/framing, bounded Tokio client, correlation, timeouts/cancellation,
shutdown/reaping/restart, health notifications and deterministic workflow/tree
reconstruction remain tested. Existing probe APIs and all pre-existing tests
are retained. G0 real Pi1.5.2 + local fake provider regression still passes.
Always await shutdown before destroying Tokio; process groups are cleanup,
not host/browser confinement. Raw text must be sanitized by renderers.

No Git/gh operations, staging, commits or remotes. Branch/SHA/tree cleanliness
NOT CHECKED under project no-agent-Git rules. No edits outside this frontend
workspace and newly created /tmp test roots; no live Pi credentials/settings/
sessions inspected or modified. Installed extension SOURCE was read-only.
Cargo.lock was reconciled through Cargo and retained; no toolchain changes.

## P03 implementation and acceptance evidence

Created: crates/pi-rpc/src/extension_ui.rs; tests/extension_ui.rs; tests/g1.rs;
crates/tui-shell/src/modal.rs and dialog_demo.rs; tests/fixtures/ui-dialogs.ts
and security-provider.py; scripts/test-dialog-pty.py; docs/security.md and
compatibility-gaps.md. Modified client.rs/lib.rs for typed one-way replies and
registry integration; tui-shell main/manifests/Cargo.lock for the isolated demo;
README, protocol baseline, architecture/dependency notes and acceptance matrix.
All pre-existing P02 tests/APIs remain. No new third-party packages were added.

### Current commands (all executed)

- PASS: cargo fmt --all -- --check
- PASS: cargo check --workspace --all-targets
- PASS: cargo clippy --workspace --all-targets -- -D warnings
- PASS: cargo test --workspace --all-targets --locked — **57 passed, 0 failed, 2 opt-in ignored**. The earlier 43-test result is the P02 baseline.
- PASS: cargo build --workspace --locked
- PASS: PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture — 1 real-Pi regression.
- PASS: PI_VISUAL_TEST_PI=/var/home/ahsan/.local/bin/pi PI_VISUAL_TEST_PACKAGES=/var/home/ahsan/.pi/agent/npm/node_modules cargo test -p pi-rpc --test g1 --locked -- --ignored --nocapture — 1 integration test containing **24 real extension cases**, with tool results AND filesystem assertions. This test passing does not override the G1 compatibility blocker.
- PASS: python3 scripts/test-dialog-pty.py /var/home/ahsan/.local/bin/pi — actual Pi + terminal PTY input: deny confirm, select second, input typed text, submit multiline prefill, restore draft, clean Ctrl+C exit. Not a Kitty/IME acceptance claim.

G1 fixture tested installed Pi1.5.2, pi-sandbox0.8.2 and @gotgenes/pi-permission-system42.0.0. Allowed read/write and bash writes pass. Outside writes, deny-over-allow hard paths, symlinks/traversal and bash writes are denied with unchanged marker files. Actual permission RPC selects were explicitly denied, canceled, timed out and approved once only inside the allowed disposable workspace. After timeout/settlement a late approval is rejected by the frontend. Combined sandbox+permission allow, allowed bash, outside denial and hard denial cases pass. No installed/live policy was read or changed.

### Reviews and corrections

Initial Astra review: separate typed UI writer items, raw option fidelity, local deadline guards, conservative invalidation, no inferred permission labels, no invented extension identities. Incorporated. Pi emits no dialog cancellation event on timeout/AbortSignal; backend ignores resolved IDs, so an exact guarantee that no late wire response can ever be transmitted is unsupported.

Final Astra review: modified y shortcuts could approve (HIGH); canceled queued approvals could still dispatch (MEDIUM); status updates exhausted seen IDs (MEDIUM); combined-extension evidence was absent. Corrected with exact modifier matching, final writer acknowledgment-cancellation checks, separate dialog-only replay IDs, and combined real-extension tests. Regression for modified shortcut was observed failing before its fix. All affected tests now pass. No additional review escalation was needed.

An initial security fixture used unverified option labels and canceled the intended approval, correctly leaving the file unchanged. Source inspection established the installed package's exact Yes/No constants; the fixture now asserts and chooses those exact values. The frontend itself never interprets option meanings. Initial PTY matching expected a full re-rendered title; terminal incremental updates required matching the changed title suffix. This corrected the test driver, not production behavior. Clippy formatting/style findings were fixed without suppression.

### P03 limits and next step

**G1 PARTIAL, P03 NOT COMPLETE.** pi-sandbox showPermissionPrompt explicitly returns abort for RPC; its native access-grant widget cannot be answered by this frontend. pi-permission-system42.0.0 selects its documented RPC fallback and works. No Pi or installed-extension patch, security bypass, automatic fallback, emergency override or live setting change was made. See compatibility-gaps.md.

UI support: select/confirm/input/editor, cancel, notify/setStatus/setWidget/setTitle/set_editor_text. All interactive dialogs have priority; no security tagging exists on the wire. Current demo shows title in-app, renders widgets as text in the notice region, and stages backend editor replacements when a local draft exists (Ctrl+R explicitly applies). Native terminal-title mutation and exact above/below widget placement are not claimed.

57 automated tests cover typed shapes, membership, deadlines, duplicate/stale/unknown IDs, simultaneous dialogs, health/crash invalidation, bounds, fire-and-forget keys, injection, editor/focus restoration, modifier guards and canceled approval behind a blocked writer. Registry phases are Pending/Queued/Consumed/Expired/Invalidated. Consumed means writer commitment, not backend receipt or authorization. Cancellation before writer dispatch prevents sending; already committed bytes cannot be recalled.

No four-mode live configuration was inspected. Installed documented allow/ask/deny behavior with yoloMode=false was tested; four-mode/emergency override behavior is NOT TESTED and no override was enabled. No real Kitty/manual accessibility/IME tests or browser confinement proof.

Do not start P04. Next: resolve the sandbox RPC grant compatibility requirement with its maintainer or an explicitly reviewed supported alternative, keeping native grants unavailable and denials effective in the meantime. Native pi remains the unchanged fallback.
