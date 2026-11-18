# Pi RPC protocol baseline

Verified 2026-11-16: installed Pi 1.5.2, requested launcher /var/home/ahsan/.local/bin/pi. No live configuration, credentials or session contents inspected. The preserved --probe-rpc diagnostic and new async library are distinct from future normal chat startup.

| Capability | P02 result |
|---|---|
| JSONL framing | LF only, optional CR; configurable async byte bound <= 8 MiB; partial/invalid/oversized/EOF handling |
| Responses | String ID/command correlation, out-of-order replies, strict known fields, unknown fields preserved |
| Lifecycle | Owned writer/readers/supervisor, timeout/cancel waiter cleanup, explicit restart/shutdown, process-group kill and reaping |
| Streaming | Text/thinking/tool argument deltas; authoritative block/message endings; no cumulative double appends |
| Tools | Concurrent toolCallId records; actual payload/status fields only |
| Agent state | agent_end remains distinct from agent_settled; retry/compaction/turn payloads retained |
| Sessions | get_entries/get_tree restoration, stable IDs, immutable entries, abandoned branches, leaf movement, cycle/missing-parent rejection |
| Extension UI | Typed select/confirm/input/editor + separate one-way replies and minimal terminal dialogs implemented; revised G1 PASS (ADR0002) |
| Shell | Direct bash rejected pending real permission-route acceptance |

Defaults: 1 MiB records, 32 pending requests, 32 event slots, two IO buffers; configured counts capped at 1024 and record size at 8 MiB. These are finite protocol budgets, not measured RSS/CPU guarantees. Stderr retains byte counts only. Always await shutdown before runtime teardown; Drop cleanup requires a live runtime. No automatic reconnect/prompt retry, frontend model worker, OS confinement or real-extension compatibility claim.

A plain pi --help against live config previously attempted a trust lock and was denied. Disposable help/probes are the supported audit route; PI_CODING_AGENT_DIR override verified in installed config implementation. No live-config escalation was used.

## P02 async foundation — 2026-11-16

Pi 1.5.2 verified against installed docs/json.md, rpc-commands.md (get_entries/get_tree), models.md and environment-variables.md. `client::Client` now supports bounded asynchronous sessions, explicit timeout/cancellation/shutdown/restart and pending correlation. `reducer::WorkflowStore` reconstructs documented content-block endings and whole-message finals, tools, retry/settled lifecycle and entry-ID branch history. 

G0 PASS: opt-in real Pi with fresh HOME/agent/project and loopback OpenAI-compatible endpoint, no tools/extensions/context discovery, PI_OFFLINE=1. Installed models.json compatible-endpoint mechanism is supported; dummy key local-test-only is test data, never a live credential. Captured stream text is local fixture; no paid provider. get_state and prompt responses match IDs; get_entries reconstructs a nonempty path; shutdown closes handles.

Run: `PI_VISUAL_TEST_PI=/absolute/path/to/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture`. Local bind/connect must be permitted; the managed sandbox initially denied it, and the authorized rerun passed. Normal suite uses deterministic Python subprocesses only. No native UI compatibility or real permission-extension claim.

Watch Client::health: malformed/oversized/truncated records mark desynchronization, requiring authoritative history reload. Event overflow closes the session. Local waiter cancellation does not send abort. Unknown compatible fields are retained; unknown block updates do not require known-variant fields. Direct bash and one-way extension UI replies are deliberately not accepted by the request method.

All current Cargo checks and exact commands are in IMPLEMENTATION-STATUS.md. Default suite: 43 pass, 1 opt-in ignored; opt-in G0: 1 pass. G1–G8 remain untested.

## P03 UI subprotocol (original evidence; scope revised below)

Installed docs/rpc-extension-ui.md and dist/modes/rpc/rpc-mode.js were checked
for Pi1.5.2. Responses: select/input/editor -> value; confirm -> confirmed bool;
cancel -> cancelled:true. IDs exactly match backend requests. Fire-and-forget
notify/setStatus/setWidget/setTitle/set_editor_text produce no response.
Normal request IDs/pending maps are separate. Unknown/malformed UI invalidates
pending interaction rather than inventing affirmative defaults. All dialogs
receive priority because no permission classification exists on the wire.

See security.md for bounds, dispatch cancellation and backend timeout races.
P03 current suite: 57 pass, 2 opt-in ignored; explicit G0/G1 integrations pass;
Original G1 was PARTIAL due to native-only sandbox grants. This historical requirement was removed by P03.1; current acceptance is defined by ADR0002.

## P03.1 scope revision

The earlier G1 PARTIAL result is historical for the original sandbox-inclusive
contract. User-approved ADR0002 removes pi-sandbox compatibility/containment
requirements. Revised G1 PASS uses Pi1.5.2 + permission-system42.0.0 exclusively,
with34 real scenarios and unchanged modal/transport regressions. Session grants
are re-prompted after process restart. Real builtin:mcp stdio tools are exercised,
including a denied call with no server execution receipt. Approved opaque code
and MCP tools can mutate otherwise path-denied markers; no containment claim.
Default Cargo result:57 pass,3 opt-in ignored (G0/current G1/archived old G1).

## P04 consumer and local ordering extension

2026-11-16: existing Pi1.5.2 protocol unchanged. Opt-in `Client::spawn_ordered`
provides `ordered_events`; `Handle::request_ordered` includes the local stdout
record boundary. These are local metadata, never new Pi commands/event fields.
Original spawn/events/request/restart behavior is preserved; consumers choose
one event feed. P04 uses this fence for get_state/get_entries hydration, preserving
unfinished streams while avoiding duplication of saved messages. Explicit new_session
and switch_session are followed by restoration; no private session-file parsing.
Direct Bash remains rejected. Rendering, wheel, mouse, local draft editing and
replay issue no prompt/steer/follow_up/bash. A deliberate connected Enter may
send prompt; Ctrl+G may send abort. UI replies use the existing separate subprotocol.

WorkflowStore default4096-entry/8MiB limits remain unchanged. P04 explicitly
selects up to10000 entries, still enforcing the8MiB atomic commit budget.
Normal startup using live user config remains outside this milestone; the only
live executable path is a disposable no-tools UI fixture. No compatibility claim
for real-user startup, automatic restart presentation or extension-native widgets.

## P04.1 normal attachment and recovery

Installed Pi1.5.2 docs/rpc-commands.md,json.md,session-format.md,extensions.md,
cli.md and dist/modes/rpc/rpc-mode.js confirm exact transition result shapes,
get_state session identity/file and retry fields. new_session/switch_session return
success with data.cancelled; cancelled:true is a veto, never successful navigation.
Initial/history loads verify the same sessionId before and after get_entries.
Successful command response sequence fences exclude old-session event prefixes;
these are local counters, never new wire fields. Normal startup keeps discovery.
Restart uses documented --session on the owned process then state/history hydration.

Real isolated tests cover two seeded v3 session trees, active/abandoned branches,
veto, rejected directory target, actual child crash, restart, new session, stable
identity and no duplicate view messages. Pi may append ordinary model metadata
on startup; tests assert exact message/branch data without assuming no such entry.
No navigation prompt/provider/tool call is issued. User-config startup is implemented
but was exercised only against disposable configuration during development.

## P04.2 ANSI and data fidelity

Pi1.5.2 RPC setStatus passes statusText unchanged; P03 still stores raw values by
statusKey. Replacement/removal behavior and one-way/no-response semantics are
unchanged. Sanitization is display-only,after JSONL and streamed-text reconstruction.
It now consumes complete ANSI sequences instead of dropping ESC and leaking suffixes.
Independent status updates are not joined as text fragments. Returned UI selections
retain their exact raw value; blank/colliding visible labels fail closed.

Normal startup/restored history never routes historical strings such as /quit
through prompt/input dispatch. Both a pure restore test and a real isolated Pi PTY
verify history display with owned processes alive. User reports both real Kitty
live commands reach ready with history/status; remaining manual checks are separate.
