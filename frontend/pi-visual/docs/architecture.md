# Architecture — P04 development milestone

**Owned process:** `pi-visual` (Rust binary) owns the cell TUI and basic user input for offline replay and an explicit isolated live fixture, while the existing unmodified Pi child owns model, tools, MCP, sessions, authorization and security hooks through the public `pi --mode rpc` JSONL protocol.

## Implemented crate boundaries

- `crates/pi-rpc`: pure parsing and serialization; classify `response` vs event, preserve unknown event fields, read-only preflight requests and response ID correlation. Includes the preserved blocking probe and the P02 async child client.
- `crates/tui-shell`: Ratatui/Crossterm transcript, local Markdown, basic input, terminal runtime and P03 dialogs. No dependence on Pi's TUI internals. Normal configured startup is implemented in P04.1; the advanced editor remains pending.

## Full planned boundaries (not shipped)

- Extended session/navigation UI beyond the implemented transport/reducer and initial history attachment.
- TUI session shell/editor, above-composer popups, Markdown cards, theme switching.
- Versioned `piui/v1` declarative widget model; local runtime and schema validation.
- 23 OpenUI component families, GenUI reactive controls, charts, diagrams and annotations.
- Native simulation core and Kitty image compositor.
- Optional *separate* Chromium sandbox/renderer with explicit opt-in and input/focus bridge.

No frontend component may directly change Pi's security policies or credentials. A visual card must never send a prompt or execute a tool without a deliberate user action. A potentially unsafe shell mode must remain disabled unless a documented Pi route preserves policy enforcement.

## Important RPC differences

Pi's native `ctx.ui.custom()`, `setHeader`, `setFooter`, `setEditorComponent`, theme instance APIs and TUI-owned autocomplete are unavailable or no-ops under RPC mode. This frontend must own its **own** UI and import only documented Pi data. No promise of automatic compatibility for pi-openui/pi-genui's TUI-native widgets.

## Data and demo truthfulness

Synthetic demo content is labeled; live cards derive only from observed backend records. A cost/context field is shown as unknown (`—`) until actual session metrics become available via Pi RPC. No tool/agent status is invented.

## P02 ownership

Client owns a supervisor; the supervisor owns Pi plus a JoinSet containing one
writer, one stdout parser and one stderr drainer. Readers never await frontend
event consumption: overflow is terminal. Writer backpressure cannot stall the
supervisor. A synchronous short-held mutex protects bounded pending oneshots;
no mutex is held over IO. Cancelled waiters remove pending entries via RAII.
Supervisor termination closes pending requests, kills the dedicated process
group, reaps Pi and aborts/joins workers. Explicit async shutdown is required
before runtime teardown. Restart preserves cleanup ownership across future
cancellation and requires an explicit state reload; no automatic prompt replay.

WorkflowStore is separate pure data reduction with clone snapshots and atomic
bounded commits. Entry IDs preserve all branches; leaf parent traversal selects
the visible branch. Raw transcript is never terminal output. Frontends must
watch the health channel and sanitize display text, and must never infer missing
tool relationships/statuses or treat agent_end as settled.

## P03 UI ownership

Shared UiState is populated by the sole stdout reader at receipt time, before
events reach the UI. Dialog responses have their own typed WriteItem and
acknowledgment; the writer validates ID/generation/deadline and canceled
receivers immediately before dispatch. Request IDs are never reused for UI
correlation. Pure registry transitions are independently tested; the minimal
modal controller owns draft/focus/edit state and renders sanitized strings.
The original P03 isolated demo uses existing P02 process ownership and contains no policy
engine, host tool executor or browser. P04 reuses its modal in the transcript.

## P03.1 security decision (supersedes earlier sandbox requirements)

Production permission interception is @gotgenes/pi-permission-system or a
separately approved replacement. No pi-sandbox dependency/fork is required or
planned. There is no OS isolation after permission approval: subprocesses and
MCP servers execute with host authority. The Rust UI/transport remains unchanged.
Original criteria/evidence are archived; ADR0002 defines revised G1, including
adversarial tests that demonstrate unrestricted approved opaque effects.
Browser confinement is a distinct future requirement; browser stays disabled.

## P04 implemented frontend boundary (supersedes starter-only description)

- `markdown.rs`: parser events -> sanitized styled rows; raw Markdown remains data.
- `transcript.rs`: stable Entry IDs/revisions, width-keyed row cache, prefix heights,
  cell clipping and logical-grapheme scroll anchor. Rendering visits visible blocks;
  cold measurement/reflow still visits the history in bounded batches. No image IO.
- `app.rs`: owned WorkflowStore, observed chronological card slots, prepared rows,
  basic draft/focus, P03 overlay and keyed extension text-state cards. Snapshots are cloned only on data ingestion or
  restoration, never for a repaint. No guessed parents/test counts/status evidence.
- `shell.rs`: offline replay or an existing ordered P02 client; typed request jobs,
  history reconciliation, bounded input/event selection and cleanup ownership.
- `terminal.rs`: reversible modes plus one polled, bounded input reader.

One Pi client is sufficient. Offline replay has none. The isolated fixture is
explicit test-only; normal startup now uses Launch without disabling discovery. No browser,
network renderer, frontend shell or second authorization engine was introduced.

The ordered-client API adds a local stdout record sequence, not a Pi wire field.
History response completion carries its sequence boundary. While hydration is
pending, input submissions are gated and up to 1,024 events / 16 MiB are buffered.
Completed pre-boundary messages are represented by durable history; unfinished
message/tool prefixes and later events are replayed. The same mechanism handles
explicit session commands, which require idle state. Initial restoration does not
invalidate freshly received P03 dialogs; actual session commands own generation
invalidation. Request completion categories prevent unrelated failures from
releasing a pending prompt guard. Overflow/desynchronization fails closed.

Budgets: 10,000 view entries, 256 KiB source per entry, 16 MiB source total;
rendered cache <=65,536 retained rows / approximately8 MiB accounted row+span
storage. These exclude allocator overhead, temporary layout and raw snapshots;
no exact RSS promise. The frontend opts into WorkflowStore's new10,000-entry
capacity; the default remains4,096 and its8 MiB serialized commit budget is
unchanged and regression-tested. Session restore resets view state, not Pi data.
Cold reflow and snapshot/reconciliation are history-size work, not O(visible).
Warm painting uses a binary prefix lookup and visible rows only. See ADR0003.

## P04.1 startup/session completion

`launch.rs` builds only an explicit Pi executable/cwd command; Client adds mode
rpc. Normal launch inherits Pi's environment without enumerating/logging secrets,
keeps extension discovery and refuses cwd inside the agent config root. The normal
transport allows8MiB records for history replies,64 queued events and the existing
pending-request bound. Worst-case queue memory is count×record size plus JSON
allocations, not an exact RSS budget. Defaults for other clients remain unchanged.

`session.rs` uses get_state/get_entries plus documented new_session/switch_session.
It verifies identity across hydration and checks idle streaming/compaction/queues
before transition. A successful transition's response boundary excludes old-session
prefixes. Veto/rejection rechecks identity and keeps the original view; ambiguous
failure disables submission. There is no session tree/file rewrite by the frontend.

Generation-tagged jobs and abort/join before restart prevent stale completions
crossing a new client. Restoration additionally checks current transport health:
a completed history result cannot revive the UI after EOF. F5 uses Client::restart,
rebinds P03 state and hydrates the saved file through Pi's --session flag. The
loaded session ID is verified. No prompt replay or unowned process termination.

Known retry/aborted-run fields have structured text; F3 exposes raw originals.
Unknown events retain safe generic details. Retained notice payloads have a separate
8MiB budget; view entries remain bounded. Code/Markdown is inert. Layout algorithm
and cache policies were preserved; only title wrapping and visible draft tail changed.
