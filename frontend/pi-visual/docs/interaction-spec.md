# P04 interaction and lifecycle contract

2026-11-16,P04.1. Automated and PTY-tested; user reports Kitty launch/readable output; other Kitty checks PENDING.

## Focus priority

The existing P03 registry is the sole dialog authority. Every pending extension
UI dialog takes priority over all transcript, draft and global keyboard/mouse
handling; there is no guessed permission classification. Enter alone cannot
approve a confirm or an unselected option. Paste cannot answer confirm/select.
Plain y/n answers confirm; arrows then Enter select; input Enter submits;
editor F2/Ctrl+Enter submits and Enter inserts newline. Escape/Ctrl+G cancels
using the original request ID. Timeout/disconnect/session-generation validation
remains in P03 and the writer rechecks immediately before dispatch.

The draft and prior focus survive interruption. Background clicks, wheel and
shortcuts are ignored while a modal owns focus. Resize is re-read by the outer
loop even while a dialog consumes input. SIGTERM/SIGHUP/SIGINT stop the runtime;
raw Ctrl+C exits outside a modal. Escape first leaves the dialog, then Ctrl+C
can exit. No terminal shortcut implicitly approves a backend operation.

## Transcript and basic input

| Input | P04 action |
|---|---|
| PgUp / PgDn; wheel | Viewport page / three-row scroll |
| Ctrl+End | Jump to latest and resume follow |
| F6 | Toggle transcript/input focus |
| Up / Down with transcript focus | One-row scroll |
| Click / left drag in transcript | Focus and row-range highlight, no clipboard write |
| Click outside transcript | Return input focus |
| Text / paste / Backspace | End-edit draft; grapheme deletion; finite paste budget |
| Enter offline | Retain draft; zero RPC calls |
| Enter connected and hydrated | Explicit prompt; clear matching draft only on success |
| Ctrl+U outside modal | Clear the basic draft locally |
| F3 outside modal | Toggle original workflow-event details; no RPC |
| F5 / explicit /restart | Restart owned normal backend and reload saved session; no prompt replay |
| Ctrl+R outside modal | Explicitly replace draft with staged backend editor text |
| Ctrl+G outside modal | Explicit Pi abort; never local tool execution |
| /new; /resume path | Explicit session RPC, then history reload; idle required |
| F2 offline | Advance one synthetic stream event (demo only) |
| F4 offline | Local demo confirm, no backend action/reply |

The draft budget is approximately 8 KiB (single typed scalar can straddle that
boundary by at most three bytes). This is an intentionally basic end-edit input,
not P09's multiline cursor/undo/Vi/completion/image/IME implementation. Long drafts
show their last two wrapped rows, including the insertion marker. Selection
is visual row highlighting only. Links are safe textual destinations; no opener
or clipboard integration. Markdown/code wraps at grapheme/cell boundaries;
wide glyphs too wide for a one-cell viewport become a replacement character.
Raw source and original tool payload remain separate from sanitized display.
Backend status/widget/title/notification text appears as local view-state cards,
with documented status/widget keys kept separate. Widget placement hints map to
this scrollable text region, not executable native components or the footer.
Very large widget bodies use bounded excerpts; original text stays in UiState.
Key-derived view IDs use local hashes; a collision fails duplicate validation.

## Scroll and failure behavior

Stable session/entry or toolCallId identities anchor visible text. Near-bottom
views follow new output; older views retain logical text position across changes
and reflow. Cold heights are measured in batches of 32 and visible rows are
clipped at both edges. No image rectangles exist yet. A disconnected/desynchronized
transport invalidates dialogs, retains draft/transcript and disables submissions.
F5 explicitly restarts the normal backend and reloads the reported saved session.
No automatic prompt replay/reconnect occurs. Restart is unavailable for the
no-session UI fixture and refuses unsaved nonempty histories. A completed history
job cannot re-enable submission after EOF; jobs carry a connection generation.

A single input thread feeds a 64-event queue. Tokio owns bounded RPC ingestion,
explicit request jobs and signal handling. Rendering consumes prepared rows and
snapshots; no locks or RPC awaits during drawing. The 20ms timer is active only
for pending reflow or modal deadlines, not idle repainting. Shutdown aborts/joins
jobs, closes/joins the input reader, awaits dedicated Pi shutdown, then restores
raw/alternate-screen/mouse/paste/keyboard/cursor modes. Poll-level panic containment
uses the same cleanup path. SIGKILL/host failure cannot guarantee restoration.

Successful navigation clears its exact submitted command only if the draft was
not edited meanwhile. Veto/failed navigation leaves the draft and old view intact;
F5 preserves unsent text. Transitions require idle streaming/compaction/queues.
Unknown outcomes fail closed and require explicit recovery. Header is a single
connection/session status row; transcript occupies h−5 rows below it, followed by
notice, two-row input and footer. It is a P04 shell, not P09/P10 composer/styling.

## P04.2 status text policy

Backend ANSI formatting is stripped as complete sequences. Ratatui still applies
its own semantic styles. No terminal escapes are forwarded,including OSC clipboard,
title or link operations. Raw originals are retained; status updates replace by key.
The compact notice can repeat a status summary but does not create message history.
Historical /quit is display-only; only live user input enters command dispatch.
User-reported live Kitty startup/history now works; the post-fix manual checklist
remains pending. Tests verify ANSI statuses plus live chrome/focus/scroll in PTYs.
