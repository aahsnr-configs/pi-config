# ADR0003: owned cell viewport and ordered history attachment

Accepted for P04,2026-11-16; real Kitty acceptance pending.

Use the existing two-crate workspace. Keep parsed Markdown, transcript layout,
app reduction and terminal runtime as modules rather than adding circular crates.
Pulldown-cmark is a parser only. No syntax engine or graphics dependency is needed
for this slice. Raw records stay in WorkflowStore; styled rows are disposable.

Cache heights by identity/revision/width and rows within finite count/accounted
byte budgets. Preserve reading position by displayed logical grapheme offset,
including when the previous row layout was evicted. Cold layout is batched32
entries; warm painting clips exact visible rows. Do not advertise constant-time
history ingestion: P02's atomic clone/serialize commit remains history-size work.

Live attachment must fence asynchronous events against the get_entries snapshot.
Add opt-in Client::spawn_ordered and Handle::request_ordered to the existing
transport, preserving all old APIs. Local counters never enter Pi requests or
stored Pi sessions. Gate prompts until hydration completes; retain unfinished
pre-fence prefixes and post-fence events. Do not create a second Pi process.

The runtime owns terminal, input reader, client and JoinSet outside the polled
body future. Catch unwind at poll boundary and await cleanup on either outcome.
This avoids detached readers/child cleanup after a body panic. Unix signal
handling and rustix O_NONBLOCK replay imports are explicit Linux-first choices.
No unsafe Rust, Pi patch, frontend tool executor or authorization policy is added.

Initial Astra review informed bounded work, event order, cache and ownership.
Final review found history races, job guard ownership, FIFO open blocking,
discarded paste, evicted anchors and panic ownership. Each received targeted
implementation and regression evidence. Follow-up fixes were tested locally;
they were not subjected to a third advisor invocation.

Removal: modules can replace parser/layout independently; ordered API is optional
and the original unsequenced client remains supported. Runtime can replace Unix
signal/import plumbing for another OS without changing protocol or reducer.

## P04.1 completion

Add small launch/session modules, keeping the transport and layout architecture.
Normal launch preserves backend discovery; disposable test-only flags never enter
normal startup. Public session commands carry explicit response fences. A veto
keeps old view/draft; ambiguous outcomes fail closed. Explicit restart reaps the
owned client, rebinds UI and verifies saved-session identity. Job generations plus
current health checks prevent stale ready state after EOF. The one P04.1 final
Astra review identified that EOF/completion race; a deterministic subprocess
regression now covers it. No additional advisor invocation was used.
