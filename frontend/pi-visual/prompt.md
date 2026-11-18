# PI VISUAL — Master implementation prompt for a production Rust frontend

> **Document type:** Self-contained, executable engineering contract for Codex. It contains the full requirements, design system, implementation sequence, and release criteria; no other requirements document or image asset is needed to interpret it. Screenshots may be provided separately by the user as optional *visual examples*, never as hidden requirements or evidence that code works.
>
> **Platform baseline:** Pi Coding Agent 1.5.2, Linux, Kitty, `~/.pi/agent` as the unchanged Pi configuration directory. Verify installed versions and current interfaces before implementation. Research baseline: November 16, 2026.
>
> **Application:** `pi-visual`, a user-owned, independent Rust executable and reproducible Cargo workspace: **Ratatui + Crossterm**, local Markdown, Kitty image integration and optional isolated Chromium browser sidecar.
>
> **Backend:** Existing, *unmodified* `pi --mode rpc`. No patches, forks, monkeypatches or changes to Pi's installed source, runtime, packages, credentials, security settings or existing sessions.
>
> **Directive:** **IMPLEMENT IN TESTED VERTICAL SLICES.** This is not a request for another plan, a static mockup or a promise of future work. Build code; exercise tests; record exact gates and limitations. Full production claims require real Kitty and security testing.

---

## 0. Mission and non-negotiable engineering contract

Act as the lead Rust systems engineer, TUI/compositor architect, Pi RPC integration engineer, numerical methods engineer, browser sandbox engineer, security reviewer, accessibility engineer and release maintainer. Produce a polished independent coding-agent frontend with **all** the user-observable GenUI/OpenUI-class functionality and a complete OpenCode-inspired chat shell.

### Functional mission

1. **Full chat UI:** Original ASCII Pi logo and clean fresh-session dashboard; upper-middle multiline composer before the first real prompt, then bottom-docked composer; in-composer model and thinking metadata, visual-only ghost placeholder, genuine `/` and `@` suggestion menus as popups above the composer, and a minimal footer showing **current directory left; estimated cost then context usage right**. No LSP, sandbox, Git branch, model or effort in the footer. Security warnings remain visible elsewhere.
2. **Card-forward transcript:** Distinct background-tinted cards with semantic accent rails for user messages, assistant Markdown, read/search, edits/diffs, shell/test results, in-progress and parallel tool calls, approvals, errors, retries and completed work. Cards reflect *actual* existing Pi RPC events; never invent progress or tool actions.
3. **Functional 23-family component system:** Stack, Text, Card, Callout, Metric, Table, Tabs, Button, Choice, Faq, Markdown, CodeBlock, Image, BarChart, LineChart, AreaChart, PieChart, RadarChart, GeoMap, Dag, Erd, InfraMap and SequenceDiagram. These must behave correctly with mouse and keyboard, responsive layouts, anchored tooltips, notes/annotations and truthful local state.
4. **GenUI-class reactive capabilities:** Local forms, sliders, toggles, selectors, calculators and safe expressions; streaming/spec validation; dashboards; charts; diagrams; Choice staging, explicit submit actions, deterministic state, serialization, and session-safe replay.
5. **Real local simulation runtime:** A draggable single pendulum, numerically correct double pendulum, accurate fixed-timestep integrator and extensible simulation engine; plus spring/oscillator, N-body/orbit, wave propagation, Game of Life, algorithms and streaming time series. Controls and frames are strictly local.
6. **Inline browser-class previews:** Secure, *user-approved*, true HTML/CSS/JavaScript/SVG/Canvas previews executing in a separately sandboxed local Chromium process. Browser pixels appear **inside a scrollable Kitty chat card**, with real mouse/keyboard interaction and focus; no external browser window for the core acceptance pendulum. The feasibility of scrolled pixel compositing is a hard gate, not a promise that the current libraries already solve it.
7. **Ownership:** No Pitago runtime dependency, no requirement to fork or install `pi-genui` or `pi-openui`, no dependence on their release schedules. Use their published capabilities, docs and examples only as *comparative references*.
8. **Reproducible production engineering:** Linux/Kitty-first, entirely local visualization, no telemetry; secure browser isolation; deterministic fixture-based tests; Pi RPC integration; CI and actionable maintenance, rollback and release documentation.

### Priority order

1. Security boundaries and no Pi modifications.
2. Correct Pi RPC semantics and real permission enforcement.
3. Zero *incremental* model requests and model-visible content for UI-only actions.
4. Correct user input, focus, streaming, session and transcript behavior.
5. Trustworthy event-derived output and compatibility with installed backend extensions.
6. Functional accessibility, performance and compositing correctness.
7. Styling and feature breadth.

### Absolute prohibitions

- **Never patch, fork, rebuild, vendor a modified version of, or monkeypatch Pi.** Do not propose a Pi patch even if a feature fails. Rework only the Rust frontend or present a strictly labeled unsupported feature.
- Do not modify `~/.pi/agent`, authentication files, user models, MCP configuration, installed extensions, `settings.json`, Pi security policies or existing sessions without a separate explicit authorization. Isolate integration testing with disposable directories and safe fixtures.
- Never use Pitago's Go runtime as a dependency, invoke its local shell, copy its whole codebase or rely on its updates. Independently implement the small Rust RPC client from official Pi docs.
- Do not introduce Node/Bun/TypeScript as the primary frontend or visualization runtime. The optional Chromium engine executes JavaScript by design; this is not permission to replace the Rust architecture.
- No background AI calls, model-based UI restyling, per-frame requests, remote browsers, hosted CDNs, telemetry, unconditional web fetches or autonomous agent actions made just to improve the presentation.
- Do not create a frontend shell that evades Pi's `pi-sandbox` or `pi-permission-system`. An explicit shell command must pass through a *verified* Pi RPC route with matching security behavior; otherwise disable that feature.
- Do not collect, store, print, transmit or scrape Pi credentials from the frontend. The unmodified Pi backend retains authentication responsibility.
- Do not make mouse clicks on decorative cards silently send prompts or perform writes; all model/host actions must be visibly distinguishable from local state changes.
- Never claim feature parity, sandbox security, browser equivalence, true terminal alpha, or zero-token results without concrete tests.

### Interpret the token requirement accurately

**Local UI-only interactions have zero additional inference calls and zero added provider-visible tokens.** This includes redraw, Markdown, simulation ticks, zoom, slider drags, keyboard movement, hover, local annotations, and HTML/JS execution of code already supplied. It does **not** mean a coding task consumes zero tokens. If Pi is explicitly asked to generate a new visualization or code, that user-requested model turn uses ordinary tokens. A *future opt-in* model-facing visualization tool also adds schema/prompt/tool-result tokens; default mode must install **no new agent-facing tool**. Never equate zero *UI overhead* with zero total Pi usage.

### Explicit feasibility qualification

Ratatui does not interpret browser HTML. Kitty supports images and animations but does not itself route HTML DOM events. Browser-backed inline interaction requires owned code for headless Chromium/CDP, frame transport, geometry, clipping, image IDs, input/focus and sandboxing. Prove this with a real scrolling-chat prototype before expanding the feature catalogue. An unsupported environment must remain useful with a native terminal fallback.

---

## 1. Mandatory discovery, research, and baseline capture

Before modifications: inspect and record `pi --version`, Pi executable resolution, `kitty --version`, `rustc --version`, `cargo --version`, terminal dimensions, color/graphics support, `$TERM`/`$TERM_PROGRAM`, current working directory, current project `Git` status, and relevant Pi configuration **paths only** (never print token secrets).

### 1.1 Mandatory research tools and authoritative sources

Use **Exa** for targeted source discovery, comparative research, issues, version changes, and security precedents **only when needed**; use **Context7** for targeted crate/library documentation lookups **only when needed**. Do not connect huge tool collections or load broad references for every coding turn. If either is unavailable, transparently report that and consult the upstream official repository/docs directly; no dependency on Exa/Context7 at runtime. For version-sensitive code, **upstream tagged code and installed Rust/Pi types override a search snippet**. Record links and dates in `docs/research-sources.md`.

Inspect at least:

- Pi CLI/RPC: `https://docs.pi.dev/latest/cli-integration`, `https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc.md`, `rpc-commands.md`, `rpc-extension-ui.md`, and `json.md` under that same `v1.5.2` docs directory. Check actual installed version as well.
- Ratatui: `https://ratatui.rs/`, `https://docs.rs/ratatui/latest/ratatui/`; Crossterm: `https://docs.rs/crossterm/latest/crossterm/`.
- Kitty graphics, unicode placeholders, scrolled image display: `https://sw.kovidgoyal.net/kitty/graphics-protocol/`; `https://github.com/ratatui/ratatui-image`; latest `https://docs.rs/ratatui-image/latest/ratatui_image/`.
- Rust editable multiline composer and IME: `https://github.com/ratatui/ratatui-textarea` and Crossterm keyboard/paste event docs. Evaluate, do not automatically depend on it.
- Rust Markdown: `https://docs.rs/tui-markdown/latest/tui_markdown/`, `https://docs.rs/ratatui-markdown/latest/ratatui_markdown/`; evaluate `pulldown-cmark`, `syntect` or `tree-sitter` only where needed.
- Chromium/CDP: `https://docs.rs/headless_chrome/latest/headless_chrome/`, official Chrome DevTools Protocol reference, and actual Linux Chromium sandbox documentation. Evaluate using a CDP client or a small audited browser process, **not** an unsafe embedded JS interpreter pretending to be a browser.
- Original capabilities: `https://github.com/chrischriscris/pi-genui`, `https://github.com/newtonapple/pi-openui`, including all 23 component docs, interaction/annotation docs, test fixtures and licenses.
- Reference only: `https://github.com/cavaldos/pitago`, especially `src/pirpc/`, `src/app/shell.go` and its session/extension UI tests. Record **what not to inherit** (independent local shell, helper extensions, implicit upstream upgrade workflow).
- OpenCode visual behavior: `https://github.com/anomalyco/opencode` (home/session/composer files). Tokyo Night palette and accessible color roles: `https://github.com/folke/tokyonight.nvim`.

Document a **dependency decision record** for each external crate: actual tested version, SemVer/API maturity, feature flags, licensing, memory/binary impact, release status, alternatives, CVE policy, removal strategy. Generate `Cargo.lock` and ensure reproducible `cargo build --locked`.

### 1.2 First compatibility audit: do not mistake Pi TUI for RPC

Confirm against installed Pi RPC implementation:

- Long-lived `pi --mode rpc` process; newline-delimited JSON in/out; correlated `id` responses; unsolicited session/tool events; asynchronous event ordering and stderr separation.
- `get_state`, `get_messages`, `get_entries`, `get_tree`, `get_session_stats`, `get_commands`, model/session commands, `prompt`, `steer`, `follow_up`, `abort`, and `bash` only when supported by the installed version.
- `extension_ui_request` and `extension_ui_response` for `select`, `confirm`, `input`, `editor`, and fire-and-forget `notify`, `setStatus`, `setWidget`, `setTitle`, `set_editor_text`.
- In RPC mode **`ctx.ui.custom()` cannot render native interactive widgets**; native Pi `setEditorComponent`, `setHeader`, `setFooter`, `addAutocompleteProvider`, `getTheme` and `setTheme` cannot be relied upon. Therefore all user-visible UI, command completion, images, themes and input handling belong to this frontend.
- Events `message_update` are **delta-only**; reconstruct streamed content without double appends. `tool_execution_start/update/end` correlate by `toolCallId`; `agent_end` is *not* reliably final when retries/compaction/follow-ups exist; settle on `agent_settled` when available.
- Make `get_entries`/`get_tree` the basis for durable session/branch reconstruction. Stable entry IDs are durable cursors; preserve append-only semantics and current branch leaf.
- `get_commands` discovers extension command names/prompt templates/skills, but built-in client controls may require a separate frontend-local registry. Never assume Pi native `@` autocomplete data is exported in RPC. Implement local file lookup with explicit security boundaries and equivalent ignore behavior where verifiable.
- Pi's custom TUI theme instances are not exposed through RPC. Do not promise automatic transfer of theme objects installed by an extension. Support external palette import only for documented, readable formats; otherwise offer an explicit frontend theme selection and optional mapping.

**Discovery output:** `docs/protocol-baseline.md`, `docs/dependency-ADRs.md`, `docs/compatibility-gaps.md`, fixture corpus of real redacted RPC lines, baseline acceptance tests and an explicit list of unsupported RPC capabilities with front-end owned solutions.

---

## 2. Technology choices and ownership boundaries

### 2.1 Chosen architecture

Implement as **a Rust Cargo workspace owned by this repository**, not a single Rust file. Suggested modules/crates (adjust names after ADR, keep responsibilities separate):

```text
pi-visual/
  Cargo.toml                 # workspace
  Cargo.lock
  crates/
    pi-rpc/                 # child lifecycle, JSONL, typed RPC, event reducer
    app-core/               # immutable event model, sessions, state machines, focus
    tui-shell/              # Ratatui + Crossterm, layout, editor, command menus
    transcript/             # streaming Markdown, tool cards, agent workflow
    ui-schema/              # safe versioned declarative GenUI/OpenUI-class spec
    ui-runtime/             # local state, controls, validation, expressions
    charts-graphs/          # charts, maps, graphs, routing, annotations
    simulation/             # fixed timestep physics, algorithms, replay
    kitty-graphics/         # pixel graphics transport, viewport clipping, lifecycle
    browser-bridge/         # OPTIONAL Chromium/CDP, isolated IPC and input
    themes/                 # semantic colors, external theme adapters
    security/               # untrusted content, data limits, browser policy
    pi-visual/              # CLI executable and runtime orchestration
  examples/                 # no-model offline gallery and replay fixtures
  tests/                    # e2e, integration, PTY/Kitty, golden/property/fuzz
  docs/                     # architecture, security, tokens, UX, maintenance
  scripts/                  # verified benchmark and release scripts
```

Avoid crate proliferation if it creates circular or costly compilation edges; enforce *logical* boundaries regardless of number of Cargo crates. All components should be independently testable without Pi/model/network where possible.

### 2.2 Render layering

1. **Ratatui/Crossterm:** terminal alternate screen, stateful editor, scrolling transcript, dialogs, keyboard/mouse, semantic cards, controls and accessible ASCII/Braille fallback.
2. **Native widget runtime:** offline charts, tables, diagrams, calculators, forms, scene graphs and simulations from a validated declarative specification. Present as part of the same transcript entry.
3. **Kitty pixel compositor:** render bitmaps/frames into clip-aware chat cell rectangles via Ratatui image support or controlled raw Kitty graphics. Must account for cursor ownership, image IDs, z-order, placeholder protocol, partial visibility and cleanup.
4. **Optional browser sidecar:** independently sandboxed Chromium executes actual HTML/CSS/JS/Canvas/SVG. Captured frame content is composited into exactly one chat widget rectangle; mouse/keyboard redirected to that widget only while explicitly focused. Browser is **local-only**, network-deny by default; no trusted host bridges.
5. **Unmodified Pi RPC backend:** exclusively owns model execution, tool calls, permissions, sandbox hooks, MCP, sessions, authorization and token accounting. No frontend model runner.

### 2.3 No unsafe inherited implementation

Do **not** fork Pitago, embed a Go runtime, port all Pitago modes or copy its shell. Its code is a reference for RPC edge cases only. The Rust RPC client is a small, typed, separately tested boundary written here. Do **not** fork Pi-genui's TUI runtime as a Rust application; implement parity against behavior and fixtures. For any copied algorithm/assets, record MIT/other license notices and upstream commit SHA; prefer independently authored Rust for maintainability.

---

## 3. Critical feasibility gates (before expensive feature work)

Treat each gate as **PASS / FAIL / PARTIAL / NOT TESTED** with repro commands, logs and real Kitty captures. A failed gate cannot be declared complete through a mock or a unit test alone. Work on independent phases may continue but the release must honestly surface missing features.

**G0 — Pi RPC correctness:** Spawn existing `pi --mode rpc` under a disposable configuration, send `get_state`, receive matching ID response, stream a fake-provider prompt, handle shutdown and broken pipes; test timeout/cancellation/restart and malformed JSONL. No changes to live Pi.

**G1 — Security/permission correctness:** With controlled fake tool and real installed sandbox/permission extensions, a denied write remains denied and a valid `extension_ui_request` is displayed and answered. No local unprotected shell is used. Run negative tests.

**G2 — Inline Kitty image in a SCROLLING transcript:** Render at least two graphics-bearing entries with independent IDs; scroll each partially into/out of view, resize and theme-change; verify no stale images, ghosting, cursor damage or text selection breakage. Verify Unicode image placeholders if supported; otherwise show exact backend limitations.

**G3 — Real interactive browser-backed pendulum INSIDE one Kitty chat:** Render user-supplied local HTML/Canvas code in isolated Chromium; show fluid pendulum frames in a bounded transcript card; drag bob; operate at least two sliders and Pause/Reset; scroll away/back, resize, switch focus to composer and back, save/reopen a *safe* snapshot. Verify no outside window and no network access. Prove all UI-only actions use zero provider calls. **This is the most important high-risk milestone.** Do not call a statically captured video a passing interactive browser preview.

**G4 — Alternative native pendulum:** Implement a native Rust physics pendulum with keyboard/mouse controls and truthful energy/period information, no browser required. It must work as fallback where graphics/browser fail and be independent of G3.

**G5 — Input/composer and above-popups:** Real multiline editor supports cursor, undo, image attachment, composition/paste, text selection and reliable `/` and `@` selection above composer, with keyboard/mouse; opening a permission dialog must interrupt and restore focus without data loss.

**G6 — Parity harness:** A sample `pi-genui` spec and an OpenUI component fixture are mapped to local schema and render fully interactively through the Rust frontend, **without either extension installed**. Do not add tool schemas or ask models to draw them for tests.

**G7 — Zero incremental LLM usage:** Fixed transcript replay, widget interaction, animation, zoom/hover/scroll, HTML preview and Markdown formatting result in exactly zero new model calls and zero model-visible UI schemas in default mode. Compare reference RPC requests and session state before/after.

**G8 — Production maturity:** Crash/panic containment, session recovery, cleanup, security review, dependency/licensing audit, accessibility, actual Kitty QA, CI, benchmarks and release artifacts. Only after this may the work be labeled production-ready.

For G3 failure, investigate within the **Rust frontend/browser bridge/Kitty protocol only**. Never patch Pi. Document precise symptom and supported degraded mode (native widget, static local snapshot, optional external explicit preview), but do not declare HTML-inline parity achieved.

---

## 4. Pi RPC client and agent semantics

Build a resilient `pi-rpc` integration as a **protocol client, not a Pi SDK reimplementation**.

### Transport, threading, lifecycle

- Launch Pi with `std::process::Command`/Tokio process using explicit arguments (`--mode rpc`, optional verified CLI flags, chosen cwd). Inherit only required environment; never log credentials; don't run through a shell string.
- Spawn stdout JSONL reader and stderr bounded log channel separately; synchronized stdin request writer; support records larger than a typical line with bounded lengths and backpressure. Reject oversized/malformed events without panicking or losing a full session.
- Correlate `id` responses by typed registry; support out-of-order replies and interleaved async events, cancellation and tombstones, per-command timeouts, read EOF, child crash, controlled restart and proper process reaping.
- Avoid holding render mutexes across IO; queue events into a bounded reducer. Rendering derives from immutable snapshots, never writes to Pi during `draw()`.
- No more than one Pi process for a normal session unless explicitly requested; no hidden model worker, summarizer or second coding agent.
- Preserve deterministic replay of recorded RPC fixtures in offline tests without spawning Pi.

### Session and turn model

- Support new/resume/rebind/tree navigation and history **only via documented RPC commands**. A feature requiring unsupported command should be disabled or implemented through safe frontend state, not a private Pi invocation.
- Reconstruct `message_start` + content-block `message_update` deltas + authoritative `message_end`, with correct per-block text/thinking/tool-call handling and deduplication after reconnect.
- Track `toolCallId` and truthful success/error/update/duration and parent relationships **only when present in actual event schema**. Do not assume `parentToolCallId` if not confirmed by the installed version. Aggregate concurrent calls accurately.
- Distinguish `agent_start`, `turn_start`, `turn_end`, `agent_end`, `agent_settled`, canceled and retrying states; never interpret `agent_end` as unconditional final completion.
- Restore saved session history from `get_entries` or `get_messages` and preserve abandoned branches/compactions correctly. Maintain stable frontend widget IDs tied to session/entry/tool IDs and schema version.
- UI-only card expansion, local search and replay must never run shell or re-read project files unless explicitly requested by user and routed through authorized APIs.
- Show genuine cost/context from `get_session_stats`/state, labeled approximate where appropriate; don't invent subscription billing amounts.

### RPC extension UI support

- Implement `extension_ui_request` (select, confirm, input, editor) as modal dialogs with exact IDs and correct `extension_ui_response` union fields, cancellation and timeout handling. Notifications/status/widgets/text requests must be namespaced so one extension cannot overwrite another's UI state.
- Security dialogs take priority over autocomplete, browser focus and background animations. Rendering a notice is never the same as granting permission. Fail closed on unrecognized permission-critical messages.
- Preserve non-visual Pi extension functionality where RPC mode permits. Record exact compatibility results for `pi-sandbox`, `pi-permission-system`, `pi-lens`, `pi-cache-hit-monitor`, `pi-feature-dev`, `pi-openui` and `pi-vim` if installed. See §12.
- RPC-origin `setWidget` generally carries text lines, not a native TUI component. Display safely without claiming full interactive compatibility. Do not forge the backend widget's input behavior.

---

## 5. Complete visual, chat and agent-workflow specification

This section is the **normative, self-contained** visual design contract. Any screenshots provided separately are optional visual guidance, not required material and not proof of implementation. Never copy another product's trademark or logos; implement an original Pi ASCII mark. Screenshot-specific fake filenames/model identifiers/metrics must never appear as factual output.

### 5.1 Theme, surfaces and visual hierarchy

**Tokyo Night reference palette** (illustrative default only, not a forced palette):

| Semantic role | Color | Required behavior |
|---|---|---|
| Terminal canvas | `#1a1b26` | Use terminal-default cells wherever possible; let Kitty control window opacity |
| Card base | `#24283b` | Visibly distinguished from surrounding canvas; filled, not outline-only |
| Elevated card | `#292e42` | Popups, focused elements, important nested information |
| Selected or hovered | `#343b58` | Focus/pointer feedback; text remains legible |
| Quiet border | `#414868` | One-cell fine line, no excessive box art |
| Primary text | `#c0caf5` | Markdown, editor text and principal labels |
| Secondary text | `#a9b1d6` | Metadata and descriptions |
| Muted text | `#565f89` | Low-priority information, never the only error cue |
| Interactive blue | `#7aa2f7` | Active search/read and selection |
| Violet accent | `#bb9af7` | Composer, user prompt, reasoning/status cues |
| Cyan accent | `#7dcfff` | Source/code and details |
| Success green | `#9ece6a` | Verified pass/completion |
| Warning amber | `#e0af68` | Approval wait, warning, incomplete work |
| Error red | `#f7768e` | Tool error, test failure, denied operation |
| Changed orange | `#ff9e64` | Edits or file modifications when distinct from success |

**Do not paint a solid full-screen rectangle:** Kitty `background_opacity` controls desktop-through transparency for terminal-default cells. Filled cards deliberately use colored cells and may be opaque. Real per-cell alpha is *not* generally available through ordinary terminal SGR styling. Implement contrast checks, truecolor fallback, monochrome/ASCII fallback and light-theme support. Choose the palette from the user-owned frontend theme system at runtime; Pi TUI theme objects are **not** exposed over RPC. Support audited import of actual accessible Pi theme files when feasible, not arbitrary dynamic themes or automatic inheritance claims.

Create shared `PanelCard`, `ProgressCard`, `SuccessCard`, `WarningCard`, `ErrorCard`, `DiffCard`, `CodeCard`, `DecisionCard`, `SummaryCard`, `TooltipCard` primitives; per-card semantic left accent rail; padded, scroll-aware content and narrow-width fallback. Avoid wrapping every text line in a separate box; group semantically coherent results. Component surfaces must contrast visibly with the canvas and each other.

### 5.2 Eighteen required UI states (V01–V18)

Each state is a standalone fixture, golden layout and actual-terminal acceptance scenario. *These descriptions are complete; no illustration file is needed.* All user/model/tool data in screenshots and fixtures must be labeled as synthetic when synthetic.

| ID | Visual state | Exact arrangement, content and behavior |
|---|---|---|
| **V01** | **Card hierarchy** | Dark default canvas; clearly brighter, filled search/edit/test/warning cards with blue/orange/green/amber accent rails; dark, independent bottom footer; visible margins between cards. |
| **V02** | **Empty-session dashboard** | Original horizontally centered ASCII `PI` mark above a composer located around 48–55% of available terminal height; no startup help, loaded-skill inventory, CLI banner or version block. Minimal anchored footer. Real warnings remain visible. |
| **V03** | **Home → conversation transition** | Left state has centered logo and upper-middle composer; after first **committed** prompt or shell/agent action, logo disappears and the *same actual editor* bottom-docks; transcript occupies remaining viewport. New/empty session restores home. Typing alone does not trigger transition. |
| **V04** | **Composer close-up** | Two-row-minimum padded multiline card, fine border, violet left rail, visual-only gray ghost text while empty, and `Chat · actual model · actual thinking effort` INSIDE the bottom metadata strip; must preserve cursor, undo, history and paste. |
| **V05** | **`/` command popover** | Elevated card immediately above composer with real available command names, descriptions, highlight/selection and bounded scrolling; keyboard, mouse, Tab/Enter/Escape correctly insert or invoke commands; never a decorative fake menu. |
| **V06** | **`@` file popover** | Matching elevated card above composer with safe, bounded local project-path suggestions; directory/type labels, truncation, navigation and correct file-mention insertion through Pi RPC; no permission bypass or full-filesystem scan. |
| **V07** | **First response and running work** | Compact user prompt card; streamed assistant Markdown; two real activity cards (read/search) showing `running` only while tool events confirm it; stable composer and footer. |
| **V08** | **Search and file read** | Grouped search/find/grep and source-file read cards with distinct accents; actual path, status, matching filename and bounded line-numbered snippet only if event data contains it. |
| **V09** | **Edit/diff** | Edited path, real unified hunk header/context, red removal and green addition rows on tinted surface; preserve original raw diff on copy and never invent hunks. |
| **V10** | **Shell/tests** | Command card with actual command/status/duration; nested green test result card with verified pass counts only when parsed unambiguously; stderr/fail results stay visible with red accents. |
| **V11** | **Parallel and nested tools** | Multiple simultaneously active cards side-by-side only at sufficient width; stable IDs and independent state. A nested card appears only when actual structured parent relationship exists; otherwise a flat activity list. |
| **V12** | **Permission interruption** | Amber waiting card and highest-priority real Pi extension dialog centered above transcript; buttons reflect actual `select`/`confirm` UI request and must send matching response only upon explicit input; browser/editor focus suspended. |
| **V13** | **Failure/retry** | Red failed command card retaining exit code and safe details; amber retry card only when a real later attempt is observed; no imaginary backoff/timeline or success. |
| **V14** | **Markdown-rich response** | Assistant Markdown block in clearly filled card; headings, bullets, quote, fenced syntax-highlighted code, aligned table and clickable safe links, responsive to narrow terminals; no model rewriting. |
| **V15** | **Completed workflow** | Green completion header, factual changed-file count and verified test count; small source/evidence cards; cost/time shown only if actual backend reports it. If no proofs, show ordinary completed text without invented metrics. |
| **V16** | **Optional subagents** | Parent role card and child status cards ONLY for observed structured agent/role metadata. If absent, omit entire tree; never infer from model thoughts or tool names. |
| **V17** | **Interactive widget inside chat** | An inline, visibly filled chart/dashboard card with controls, focus and hover; user can interact locally while editor and real transcript remain usable. Widget is implemented by Rust/optional browser, *not* by an assumed Pi TUI widget. |
| **V18** | **Narrow/short terminal** | At 60×18 or less, cards stack, popup fits available rows and composer/footer retain input accessibility; scroll indicators and clipped content remain discoverable. |

### 5.3 Home, composer, popups and footer mechanics

- **Home/chat phase:** `STARTUP→HOME` when meaningful current branch is empty; `STARTUP→CHAT` for resumed branch; `HOME→CHAT` only on *committed* prompt or genuine action; `CHAT→HOME` on `/new` or branch change to empty; re-evaluate after session switch/restore, resize or reconnect. Do not trigger transition for typing, autocomplete, theme change, `/help` or other UI-only command.
- **Editor:** Real buffer with multiline growth, correct Unicode grapheme columns, paste, selection, history, undo/redo, safe clipboard, image attachment where supported, IME-aware fallback and Kitty enhanced keyboard handling. Ghost text is **never** inserted into the editable value or RPC payload. Style editor with filled background and accent rail.
- **Metadata strip:** Actual mode, actual model identifier and actual reasoning level if known; unknown values are marked `—`. Model/effort must not repeat in footer.
- **Popup:** Both `/` and `@` use one focused, elevated overlay above composer (not below); preserve matching, selection, scroll, insertion and cancel. `/` uses verified `get_commands` data plus explicitly implemented frontend commands. `@` uses an independently bounded, read-only local path index; do not claim it automatically shares Pi native TUI's search rank or ignores. Test exact mention semantics against the installed Pi RPC backend before release.
- **Minimal footer:** Left current cwd (home-shortened, safely ellipsized), right `Cost <reported/estimated or —> · Context <used % and window or —>` in that exact order. Cost is API-equivalent estimate, *not* claimed subscription charges. Never show LSP, sandbox, model, effort, Git, spinner, tool inventory or shortcut hints in the footer. Security alerts remain elsewhere.
- **Transparency:** Unpainted canvas must use Kitty default background; filled cards intentionally contrast. Never change Kitty opacity, OSC palette or remote-control settings. Test opacity 1.0 and 0.88 without requiring changes.
- **Rendering ownership:** The Rust frontend owns the entire TUI. It **does not** call Pi `setHeader`, `setFooter`, `setEditorComponent`, `setWidget` or `registerToolRenderer`; these are not a way to customize the Rust RPC frontend.

### 5.4 Truthful agent workflow renderer

Design a `WorkflowStore` keyed by session/entry/`toolCallId`. Use only observed Pi RPC events; persist bounded view state separate from stored Pi messages. `message_start`/`message_update`/`message_end` control streaming text; `tool_execution_start`/update/end control execution cards; `agent_end` is not settled, while `agent_settled` is final when supported. Preserve progress/error evidence on interruption, replay and branch changes. Distinguish aborted/permission-blocked/failed only when backend data proves it.

- **Read/ls/find/grep:** Surface existing path and bounded result excerpt. Never run additional file reads to decorate cards.
- **Edit/write/diff:** Render only actual result hunks and paths. Maintain raw-copy fidelity.
- **Bash/tests:** Show exactly observed command, exit status/duration and conclusively parseable test summary. No model- or renderer-triggered shell actions.
- **MCP/unknown tools:** Generic safe named cards; don't assume undocumented result schemas.
- **Permission and sandbox:** Modal ownership belongs to real RPC extension UI request; never fabricate approval, override, denial or confirmation.
- **Parallelism and nested tools:** Only display relationships present in the actual protocol or explicitly supported extension payload; avoid invented parent IDs or subagent roles.
- **Final Markdown and summaries:** Render locally without LLM rewriting. Verified completion statistics only where evidence exists. Preserve original transcript data, raw outputs and session semantics.

### 5.5 Transcript, Markdown, accessibility and Vi mode

- Cache cell height and render only visible transcript blocks; correct partial clipping for text and Kitty images, image placement cleanup, scroll anchoring and jump-to-latest. Streaming must not jump while user examines earlier output.
- Use a local Markdown parser/highlighter. Support CommonMark/GFM-like headings, code fences, syntax highlighting, safe links, tables, task lists, quotes and emphasis; preserve raw markdown in session data and never use a model for styling. Do not execute fenced HTML/JS automatically.
- Unicode display cells, tabs, combining code points, CJK/emoji, ANSI/OSC stripping, terminal selection/copy and accessible keyboard alternatives must be tested. Every interactive widget needs focus/selection cues and textual data alternatives.
- Provide *frontend-owned* Vi insert/normal mode for users familiar with `pi-vim`, with explicit Escape priority: active security modal → widget/browser focus → editor mode. Preserve tested `Ctrl+G` interrupt through Pi RPC; don't assume Pi's native `pi-vim` hooks apply over RPC.
- A responsive text-only mode must remain available if Kitty pixel support, optional Chromium or mouse reporting is absent. Never trap the user in a widget without an obvious exit.

---

## 6. Complete pi-openui-class functional parity: all 23 families

The requirement is **observable feature parity**, not a port of OpenUI Lang and not syntactic identity with its upstream functions. The following **23** component families are required as a self-contained capability contract; document each with source behavior, local API, implementation, keyboard/mouse actions, fixture, and independent parity evidence. Names below are semantic requirements and must all be covered.

| # | Family | Minimum full Rust behavior and acceptance |
|---|---|---|
| 01 | **Stack** | Vertical/horizontal layout, gap/padding/wrap, nested constrained alignment, overflow rules and narrow screens. |
| 02 | **Text** | Styles, text alignment, wrapping, selectable/copyable content and Unicode-visible-width handling. |
| 03 | **Card** | Color-filled framed panels, optional headers, nested component trees, semantic accents and theme inheritance. |
| 04 | **Callout** | Info/success/warn/error, compact semantic colors and screen-reader/text equivalents. |
| 05 | **Metric** | KPI/value/change/trend tiles, formatting/units, hover/focus details and no fictional computed data. |
| 06 | **Table** | Column sizing, sorting/filtering, horizontal+vertical scroll, pinned columns, selected cell/row, full truncated value inspection, annotations; 100×16 baseline stress fixture. |
| 07 | **Tabs** | Mouse/keyboard switching, state retention, accessible focus, responsive labels and nested content. |
| 08 | **Button** | Local action/disabled/loading/selected and *separate explicit* send-to-agent action; never default-send a model request upon harmless UI interaction. |
| 09 | **Choice** | Single/multi questions, optional Other, staging of multiple answers and **one user-confirmed send** when explicitly requested; Clear, retry and sent-lock handling. |
| 10 | **Faq** | Expand/collapse multiple items, per-entry Markdown, keyboard navigation, tooltips and notes. |
| 11 | **Markdown** | GFM-like rich terminal rendering, responsive tables/code, fenced syntax, safe local image captions and links. |
| 12 | **CodeBlock** | Syntax highlighting, selectable source, copy, optional line numbers, wrapping/horizontal pan and missing-language fallback. |
| 13 | **Image** | Local authorized file or user-supplied data image, actual Kitty pixels where available, dimensions/caption, scaling, tooltip, offline fallback; never arbitrary network fetch. |
| 14 | **BarChart** | Horizontal/vertical, multiple values, axes/legend/labels, hover/focus values, accessible data table. |
| 15 | **LineChart** | Multiseries, axes/legends, interpolation/smoothing policies, crosshair, tooltip, Braille/native plotting, bounded downsampling. |
| 16 | **AreaChart** | Genuine filled/stacked area with mathematically correct baseline and totals, legend, tooltip and accurate labels; not a plain LineChart alias. |
| 17 | **PieChart** | Pie/donut/semi-circular where justified, slice proportions, percent/legend, mouse+keyboard focus, text alternative. |
| 18 | **RadarChart** | Multiseries multi-axis polygons, scales, legend, focusable points/axes and accessible table alternative. |
| 19 | **GeoMap** | Offline bundled geography (license checked), accurate projection and fit, pins/links, zoom/pan, annotations and keyboard pin list; offline by default. |
| 20 | **Dag** | Layered directed graph, arrows, cycles, route crossings, label overlap handling, node/edge focus, annotations and routing oracle. |
| 21 | **Erd** | Entity tables/fields, cardinality connectors, linking/highlight, overflow navigation and semantic structure. |
| 22 | **InfraMap** | Zoned service/database/queue/compute nodes, directed labeled links, state colors, routing and selection. |
| 23 | **SequenceDiagram** | Participants, chronological messages, arrows, note blocks, alt/loop frames, horizontal+vertical pan and accessible event list. |

**Cross-cutting parity is mandatory, not a later enhancement**: anchored hover/focus tooltips, semantic pinned notes/annotations on text/chart/table/diagram/map/image elements; local annotation editing and explicit copy/stage-to-composer; no auto-sending; correct disabled states; mouse+keyboard parity; responsive 40/60/80/100/120/160 columns; focus/scroll indicators; full value inspection; live table sorting/filtering; clipboard safety; Unicode graphemes; image lifecycle; schema migration; bounded parsing and rendering.

**Graphs:** Implement deterministic routing independent of styling. Include cycle detection, fan-in/out, self-links, crossings, accurate cardinality arrows, labels, fallbacks and an **independent round-trip graph routing oracle** that detects missing/ambiguous edges. Property-test randomized graph layouts. Do not silently drop edges to make the diagram look nicer.

**Annotations:** Notes are stored locally and attributed to widget/spec element IDs, not written into the model transcript; sending a note to the agent requires an intentional, visibly labeled submit. Sanitize controls, fake mentions and ANSI/OSC injection. Have per-session and per-user persistence controls.

**Capability proofs:** Create a no-model `/visuals gallery` with **23 fixtures**. For each component, require valid schema, snapshot/golden, mouse and keyboard test, resize test, accessible fallback and round-trip serialization. The final app must work with `pi-openui` uninstalled/disabled. A component name with a placeholder is **not** parity.

### Beyond 23 families: preserve GenUI features

Also implement these GenUI-class behaviors: basic `col/row/box`, `divider`, notes, key/value, stat, table, tree, flow, progress, heatmap, multi-select, sliders, toggles, text input, local calculators, streamed structured-spec validation, bounded expressions, tabs, form validation, local input binding/recomputation, responsive mouse hit-testing, explicit wait/non-wait user workflows and saved-state behavior. Do not regress existing GenUI-equivalent experiences just because another OpenUI component overlaps.

Some old GenUI constructs are **behavioral concepts**, not mandatory identical JSON names: implement the simplest safe idiomatic version in `piui/v1`, with a documented compatibility mapping. A `pi-genui` parser and an optional OpenUI Lang parser should be isolated adapters, never two competing runtimes.

---

## 7. Safe versioned declarative specification and runtime

### 7.1 Primary local spec, no permanent model schema

Design a versioned, serialization-safe schema (`piui/v1`) separate from Ratatui, Chromium and Pi. The data model should have:

- `id`, `schemaVersion`, `kind`, optional `title`, `layout`, named element children, stable element keys and explicit interaction/action semantics;
- typed `DataSource` (`inline`, authorized local file, existing tool result, current session record, bounded local stream, user-selected dataset), validation/units, transforms and sample/test data;
- `State` for text inputs/tabs/selections/physics parameters, typed computed `Derived` expressions, and `View` with adaptive layout;
- local-only `Action`s such as set/toggle/step/reset/zoom/pan/copy/stage; **agent-facing** actions must be separate `AgentAction` requiring explicit human confirmation and clearly documented token implications;
- semantic colors/tokens, accessibility labels, mouse/keyboard targets, capabilities, sizing/limits, trusted-origin metadata, lifecycle/persistence policy and data ownership.

Example **illustrative schema, not a preverified final API**:

```json
{
  "schema": "piui/v1",
  "id": "demo-pendulum",
  "kind": "simulation",
  "engine": "native-pendulum",
  "title": "Interactive Pendulum",
  "parameters": {"gravity": 9.81, "length": 1.5, "angleDeg": 42, "damping": 0.04},
  "controls": ["play", "pause", "reset", "step", "drag-bob"],
  "presentation": {"size": "medium", "theme": "inherit"},
  "agentActions": []
}
```

Implement clear validation errors with source location/path, bounded recursion, element count, strings, series/point counts, map/graph counts, image byte sizes, runtime memory, frame count, network state and CPU budget. The upstream `pi-openui` limits (approx. 100 KB source, 1,500 elements, depth 16, 100×16 tables, 200 points/8 series, 60 DAG nodes) are **research reference thresholds**, not automatic Rust defaults; benchmark and justify the local limits. Define `serde` serialization, backwards-compatible migrations, unknown version fallback, canonical encode/decode and full fuzz/invalid-input tests.

**Default ingest paths (zero new model-facing tool schemas):** explicit user `/visuals open <path>`; a user-approved structured `piui` fenced code block in the existing assistant transcript; previously emitted tool results with explicitly recognized JSON envelope; local offline gallery; verified read-only importer for old saved widget specs. **Never automatically run JavaScript because any code fence says `html` or `javascript`.** Preview execution always requires a distinct trust gate.

**Optional model-directed mode:** if the user explicitly wants Pi to create a `piui` spec, support authoring via ordinary message/code output (normal model tokens) or, only after separate review, a single small opt-in Pi-side tool adapter. It must not be enabled by default and must separately benchmark tool-schema additions, request payloads and historical context growth. No model-backed slider events or animation frames.

### 7.2 Reactive state engine

- Implement deterministic local reducer/event loop, typed state transitions, validation-before-commit, UI invalidation and time source injection.
- Separate **frame rate** from fixed **simulation step**; support play/pause/step/reset/seek/speed/trace, bounds, state checkpoints and disposal. No unbounded event arrays.
- Derived computations should use a safe expression interpreter (e.g. small arithmetic/Boolean/math grammar with strict whitelists), not `eval`, `node:vm`, `rhai` with arbitrary I/O or subprocess evaluation by default. `pi-genui`'s 150 ms `node:vm` timeout is explicitly *not* a security boundary; do not duplicate that mistake.
- Local formulas may use unit-safe arithmetic, aggregates, range/clamp/min/max, chart transforms, progressive updates and fields. Reject prototype-pollution-like malicious paths, NaN/Inf, exponential growth and invalid indices.
- Stable widget identity survives transcript scroll/repaint. Pause offscreen work; resume only when visible and permitted. Restore only safe serialized state after relaunch, not live JS heap or undisclosed browser sessions.
- Never append animation steps, hover events, full datasets or screenshots to Pi session/model context.
- Keep a single schema/validation source shared by native engine and browser adapter; browser is a render backend, not a second truth for data/state permissions.

### 7.3 Streaming and data updates

- Accept partial streamed JSON tool arguments and assistant code fences **for visual preview only**; do not execute unvalidated/incomplete actions. Accumulate with size/time limits. Show neutral `loading` skeleton until a valid recoverable subtree exists; never display fabricated content.
- Distinguish replacement vs incremental updates by stable message/widget ID; never duplicate a widget when an RPC final message supersedes streaming deltas.
- Support multi-series charts and local data streams with bounded backpressure, downsampling and paused/offscreen throttling. No time-series updates to the model unless explicitly requested.

---

## 8. Native Rust simulation engine and numerical accuracy

Implement a **general scene/simulation kernel**, not 15 unrelated one-off animations. Use testable pure Rust physics functions separate from the terminal painter and from browser frames.

### Scene and controls

- Scene primitives: point, arc, path, polyline, grid, axes, arrows, labels, discs, polygons, phase plot, world-to-screen transforms, pan/zoom, hit testing, clipping, optional particle trails and sprites.
- Backends: Unicode/Braille/blocks always available; Kitty pixel graphics where proven; browser canvas when opted in. Text-only accessible description and parameter table for every simulation.
- Scheduler: fixed timestep with bounded catch-up and optional interpolation; idle/offscreen FPS reduction; headless deterministic replay; user-defined speed and reset; errors don't kill the terminal loop.
- Provide direct parameter inputs and drag manipulation with pointer coordinate testing, keyboard substitute and undo/reset; no model requests for any local action.
- Use mathematical/numerical invariants as tests where appropriate: energy drift, convergence under timestep reduction, known equilibria, conservation/stability, reproducible random seeds and no NaNs for invalid input.

### Required baseline demonstration: pendulums

1. **Single pendulum** matching the interactive ChatGPT-style example's behavior: gravity, length, initial angle, damping, Play/Pause/Reset, drag bob, visible energy and period approximation (labeled *small-angle*). Use nonlinear pendulum ODE and an appropriate fixed-step numerical integrator (e.g. RK4); mention approximation limits and units.
2. **Double pendulum** with two masses, two lengths, gravity, both initial angles/velocities, optional damping, adjustable playback, live trail and phase/angle/energy charts, Pause/Step/Reset/Clear Trail. Coupled equations verified by independent reference and energy drift tests. Never call a static gif 'interactive'.

### Required generalization (all demonstrable, not only roadmap headings)

3. Spring-mass/damped/driven harmonic oscillator and live energy/displacement plots.
4. N-body/orbital dynamics with configurable particles, masses/forces, stable integration and trails.
5. Wave propagation/vibrating string discretization with speed, damping and boundary controls.
6. Conway/Game of Life with direct grid edit, pause/step/presets and deterministic update.
7. Algorithm animation: at least sorting **and** pathfinding with step/speed, operations highlight and accessible trace.
8. Streaming local time-series/multivariate dashboard with zoom, tooltips, crosshair and pause.

**Stretch only after mandatory gates:** phase portraits, attractors/Lissajous, Monte Carlo, random walks, fractals, 2D rigid-body collision, particle systems, network process animation, Gantt/timeline, 3D wireframe/surface projections, audio-independent visualizations and exports. Don't label a stretch feature complete if only an interface stub exists.

### Performance methodology

Measure real Kitty frame pacing, terminal traffic, CPU, memory, jitter and input latency on actual hardware. Native Braille at roughly 10–20 FPS may be a first measurable target, not a guarantee; pixel/Chromium rendering targets must be set **after G3 proof**. Do not assert 60 FPS without measurement. Coalesce frame invalidations; bound transfer bandwidth and image caches; prevent animation work during hidden/offscreen/paused states. The composer must remain responsive while multiple background cards are visible.

---

## 9. Inline HTML/JavaScript/CSS/SVG/Canvas: browser-backed feature contract

This is the key expansion beyond ordinary `pi-openui`/`pi-genui` and must never be quietly replaced by launching Chrome in another OS window.

### 9.1 Two independent rendering modes

**Static/trusted render-only:** HTML snippets can be shown as escaped code and sanitized preview where supported; SVG can be parsed/rasterized with explicit sanitizer; image files can be rendered locally. This mode executes **no JS**, requires no Chromium and remains functional on disconnected/offline Kitty.

**Interactive HTML/JS browser mode (opt in per untrusted artifact):** use Chromium controlled through CDP from a sandboxed helper process. Render element/viewport frames, transfer to Kitty images in an inline, scrollable chat card, map mouse coordinates, wheel, keyboard/focus and select/drag interactions to a browser-local viewport. Show clear *Run interactive preview* affordance and block by default until explicit consent. No silent JS execution from assistant text.

### 9.2 Browser bridge ownership and isolation

- Define a `BrowserSessionId` and chat `WidgetId` pair; one framed origin/isolation boundary per untrusted HTML widget. Support multiple widgets without cross-widget state leakage.
- Launch a real Chromium executable using a sandboxed child, **never with `--no-sandbox` for convenience**. If user namespaces/seccomp/namespace prerequisites prevent safe sandbox startup, mark browser mode unavailable and keep native/static fallback. Do not weaken host security.
- Prefer bubblewrap/user namespaces/seccomp/process resource limits where available and audited; run browser with unprivileged UID, read-only filesystem, isolated ephemeral profile, deny `file://` and nonapproved navigation, no host mounts/IPC sockets, no clipboard/system APIs exposed to content and no Pi/API keys in environment.
- Network and remote resource fetching **disabled by default** including images, fonts, webfonts, websockets, fetch, Service Workers, `data:`/`blob:` escape cases and downloads. Implement an auditable allowlist only after explicit user approval. CDN dependencies never auto-fetch.
- Expose only narrowly typed local UI actions through audited IPC; never arbitrary shell/host API, unrestricted `evaluateJS` bridges, page access to Pi RPC or automatic 'send to agent'. User-approved typed state return is separate.
- Cap CPU, RSS, open processes, GPU usage, frame rate, dimensions, duration, input events, resource size, JS execution time and widget count. Kill/offload crashed/infinitely-looping widgets without killing the Pi child/frontend.
- Treat MIME sniffing, HTML sanitization, CSP, navigation, browser sandbox, resource intercept and domain isolation as defense in depth. CSP alone is not a sandbox. Keep logs redacted.
- Pin browser minimum/version range and CDP compatibility; use system Chromium only after verification or a well-defined optional managed runtime, never a surprise auto-download during Pi sessions.

### 9.3 Compositing into terminal chat

- Bind each browser frame to a stable widget cell rectangle. Use Kitty Unicode placeholder protocol if the terminal supports it; otherwise a proven placement/clip method. Do not write uncontrolled graphics escapes from an arbitrary async thread while Ratatui is drawing.
- Track pixel ratio, terminal cell size, viewport scale, scroll offset, visible crop, overlays, z-index and fixed composer. A mouse event is delivered to the exact widget/card that owns its hit rectangle, with per-widget local transformed coordinates.
- Explicitly manage focus transitions: editor -> browser -> menu/dialog -> editor. Esc closes browser interaction (not always browser content), Ctrl+G honors interrupt path when context is appropriate, untrusted widget cannot trap focus permanently.
- Honor scrollbar/trackpad/wheel events: focused widget may scroll its own content; otherwise transcript scroll. Clicking outside returns focus. Text cursor and copy behavior outside the widget cannot be broken by graphics protocol state.
- Suspend Chromium rendering when widget is outside viewport or tab inactive; resume and rerender upon reveal; delete image IDs/release memory on widget disposal, session switch, project change and app exit.
- Provide crash/restart overlay showing truthful error, not black rectangle. Any browser action requiring real network or host file access must be user initiated and governed by a separate authorization gate.

### 9.4 Hard acceptance scenarios

- The user-provided interactive pendulum HTML runs **in the same chat card** with draggable bob, working sliders, pause/reset, accurate simulation updates and zero model calls during interaction.
- Two independent HTML/JS widgets coexist; dragging the first does not change second's state; scrolling, partially clipped display, resizing and clicking outside cause no image overlap.
- Show interactive plots from locally bundled D3 or Plotly only if security/license/build audited; otherwise implement an equivalent native chart without silently downloading libraries. HTML code built for React/Vue/Svelte must be explicitly built from local trusted dependencies, not executed by a raw browser with remote CDN.
- Handle Canvas and SVG interactivity. WebGL/Three.js and heavy 3D should be tested and **labeled best effort**; graphics acceleration, screenshot cadence and CDP overhead may prohibit web-app parity. No false promise of universal WebGPU/WebGL support.
- A malicious HTML sample attempting `file://` reads, network fetch, clipboard, host shell, endless loop, huge canvas and navigation outside sandbox is denied or killed. Browser mode cannot access Pi config/API tokens.
- Screen recording and screenshots show *actual* Kitty inline behavior with window title and visible transcript; don't substitute a browser screenshot placed on top of a static app mock.

### 9.5 Portability of ChatGPT/Claude/Codex web-app snippets

The requested user experience is **inline interactive browser-class output**, not a guarantee that proprietary web-client widget formats can be executed unchanged. Accept genuine self-contained, standards-based HTML/CSS/JavaScript, SVG/Canvas apps and locally built trusted bundles under the browser security policy. If a snippet references ChatGPT/Claude/Codex-specific runtime services, `AppBlock`-style data bindings, hidden JS globals, proprietary action APIs, external component registries or hosted assets, **do not silently emulate those unknown interfaces**. Offer a documented local adapter for verified public bindings or request an explicit standalone export. Keep the data conversion and any host callbacks typed, consent-controlled, offline and unable to reach Pi credentials. Test a simple `canvas` pendulum, a standard SVG animation, an interactive chart, and a snippet with unavailable proprietary globals that fails gracefully and informatively.

### 9.6 Browser-free degradation

If Chromium cannot launch safely, **the normal chat and all 23 native component families remain functional**. Existing HTML preview shows source + a clear optional static fallback, not an automatically opened window. A manual explicit `Open external browser` option may exist for troubleshooting but does **not** count toward the inline-browser requirement.

---

## 10. Tokens, context and model independence: hard measurement contract

**Default product adds no model-facing tools, system prompts, AGENTS text, provider calls or MCP requests.** The frontend is only a local RPC UI. Pi's existing tools and installed extensions may already affect model context; distinguish baseline costs from **marginal costs caused by this product**.

### Mandatory zero-token actions

The following always use **zero extra model requests and zero model-visible tokens** (verify automatically): open/resume previous widget; native/image/Markdown render; resize/scroll/pan/zoom; hover/crosshair/tooltip; select tab/table row; sort/filter locally; drag a pendulum; simulate physics time; Pause/Step/Reset; chart animation; change theme; select/copy text; inspect saved data; annotate locally; browse a saved offline gallery; browser-side JS event that stays local; browser image frame/screenshot/IPC update. Zero network model calls during any such action.

An explicit **send-to-agent** button is not a UI-only action: after user confirmation it can create a legitimate Pi prompt and therefore token usage. Clearly label it and display the final composed input before sending. Never send on incidental click, hover, focused slider change, or note pinning.

### Prove no default prompt/tool overhead

- Capture byte-exact or normalized provider request snapshots in deterministic fake-provider integration tests with vanilla Pi and with this frontend. Same tool manifest and model-visible messages must result for the same user action; only the transport/GUI changes.
- Capture frontend-to-Pi `prompt`, `steer`, `follow_up`, `bash`, and any UI-dialog responses. For non-agent UI-only actions there must be **no new `prompt`/`steer`/`follow_up`/`bash` call**.
- Never inject generated HTML/frames or complete widget state into `get_messages`/session notes. Save state separately in explicit frontend storage. Optional output to user must be an explicit user command.
- Cost/context footer should read existing reported usage via RPC, without causing model turns or changing model selection.
- If an opt-in schema authoring tool is added later, benchmark schema tokens at load; single render, 1/10/100 histories, session resume, and prompt caching. Clearly label that mode **NONZERO OVERHEAD**; it cannot be part of the default zero-tool release.
- Rendering CPU/GPU/Chromium usage is **not** LLM token consumption; still benchmark resource/battery/latency separately.

Produce `docs/token-budget.md` with baseline config, exact commands, sample RPC traces, input/output tool-schema size, model calls delta, context bytes and repeatable independent verification. Do not invent costs for subscription models. No hidden AI summarizers or per-frame agents.

### Coding-agent implementation token efficiency

- Work in small, bounded vertical slices; at each stage give Codex only the relevant crate files, targeted README/ADR and current failing tests rather than reprinting this entire master specification repeatedly.
- Maintain short `docs/IMPLEMENTATION-STATUS.md` and machine-readable acceptance matrix. Keep high-level architectural contracts discoverable via code comments/interfaces and small design ADRs. Avoid 100 KB prompt instructions auto-loaded into every coding task.
- Strictly avoid unmeasured library migrations, wholesale Pitago ports and broad speculative refactoring. Compare Rust dependencies with **current Context7/upstream docs**, not memorized examples, to prevent compilation-repair cycles.
- Don't claim a numerical expected Codex token saving versus Go/hybrid; measure actual development usage if telemetry is available and opted in. Record spent effort per milestone, failures/rework and compilation speed. User's subscription limits and billed API usage must not be guessed.

---

## 11. Security, privacy, permissions and trust boundaries

Threat model includes: user-supplied/assistant-generated HTML/JS, model output text, malicious Unicode/ANSI/OSC in file paths/tool results, huge chart datasets, image decoders, expression injection, browser escape, browser navigation/network, filesystem symlinks, CSRF-like local IPC, clipboard exfiltration, independent shell escalation, TTY escape abuse, prompt injection, session confusion, and untrusted asset licenses.

### Host vs Pi sandbox

`pi-sandbox` and `pi-permission-system` protect their configured **Pi tool calls**, not all Rust frontend processes. Do not assume they protect `pi-visual` or its Chromium child. Frontend file imports are read-only and explicitly authorized; browser runs in its own stricter sandbox. Rust native physics never needs shell or external network access. Permission decisions remain Pi-owned.

### Hard runtime security requirements

- Never read or display `~/.pi/agent/auth.json`, API tokens, SSH keys, shell history, arbitrary private files or raw environment variables. When user explicitly imports a file, validate canonical path, symlink traversal, file size and type; no automatic project-wide scanning for visuals.
- Sanitize all untrusted text before writing to terminal; neutralize control and bidirectional spoofing where appropriate; prohibit terminal/clipboard OSC injection. Preserve original raw text for user-approved copy, not direct terminal execution.
- RPC stdin/stdout are child process pipes, not untrusted external sockets; still parse strictly, bound buffers and redact errors. Do not open a listening TCP server by default. If a browser sidecar uses IPC, use an authenticated/permissioned Unix socket or inherited pipe, bound messages, and peer validation.
- Segregate trusted local author assets from model-generated untrusted content. HTML/JS never gets trusted host bridges. All browser permissions and external links require active user action; network is off by default.
- Every **frontend-initiated new** model/agent execution requires an explicit user action and passes through Pi's normal RPC. Pi may continue tool calls, retries or queued work as part of an already authorized agent run; do not block or silently change those backend semantics. No frontend parallel shell, subprocess analysis, extra `codex` invocation or hidden tests for decorative displays.
- Keep filesystem and runtime state storage under a new documented frontend cache/config location; never override Pi files. User-specific cached widgets may contain sensitive project data; apply owner-only permissions and provide purge controls.
- Bound renderer memory, image frame sizes, maximum browser count, CPU/GPU, frame time, timeouts, expressions and archive decompression; enforce cancellation and cleanup on session change, app exit and process crash.
- Dependency audit: `cargo audit` (or supported equivalent), `cargo deny` for licenses and disallowed sources, reproducible lockfile, minimum supported Rust version, pinned supply chain. Browser binary updates must be governed and version-tested.
- Keep all telemetry opt-in and local only, if present. Never send rendered content or private filenames to Exa/Context7 in a user runtime. Those are development research providers, not installed UI dependencies.

### Trust-flow documentation

Provide `docs/security.md` with dataflow diagrams, OS process permissions, browser flags/profile and IPC contract, supported/denied origins, filesystem mount policy, permissions precedence, escape handling, token/credential handling, incident response, disable switch, and security test evidence. Include explicit conditions under which **interactive HTML must be disabled** (no secure process confinement, unsafe GPU/browser flags, unsupported compositor, untrusted external asset request). Don't call a soft JS timeout a sandbox.

---

## 12. Existing Pi extension compatibility and migration

The new Rust frontend is a replacement **for Pi's interactive presentation layer only**. Users continue using the **same Pi executable/config, models, sessions, MCP, skills and backend tools**. Pi loads extensions in RPC mode according to its own rules; native Pi TUI UI hooks are not available. Therefore make no unconditional promise that every UI-specific extension works unchanged.

| Existing or possible extension | Rust frontend requirement | Unavoidable distinction / test |
|---|---|---|
| `pi-sandbox` | Preserve actual access enforcement; show requests/errors; never change policies | Verify real allow/deny behavior through Pi RPC; browser/frontend separately sandboxed. |
| `pi-permission-system` | Render `select`/`confirm`/`input` extension UI requests with correct responses/timeouts | No autoallow; test all 4 policy modes if configured, including required emergency overrides, without changing user security settings. |
| `pi-cache-hit-monitor` | Surface backend-reported textual notifications/status/widgets if present, elsewhere than minimal footer | Its native TUI custom component will not automatically transfer. Don't fabricate cache metrics. |
| `pi-lens` | Preserve LSP agent/tool functionality where available and show actual diagnostics/notifications | No LSP in minimal footer; don't pretend custom TUI renderer survives RPC. |
| `pi-feature-dev` | Skills and commands should remain discoverable through Pi RPC where supported | Test `/` command discovery and execution rather than assuming Pi TUI autocomplete behavior. |
| `pi-openui` | Capability parity delivered by new Rust native renderer, **without it installed** | When installed for migration, native interactive `render_widget` cannot be assumed functional in RPC. Never silently unregister or delete it from live config. |
| `pi-genui` | Capability parity delivered by Rust spec/runtime; no hard dependency | Its upstream interactive input is explicitly unavailable in RPC. Do not claim its `render_ui` tool supplies working original TUI widgets in Rust. |
| `pi-vim` (if installed) | Implement user-visible Vi editing in Rust frontend | Pi's TUI editor override is not active in RPC; test Escape/normal/insert and `Ctrl+G` semantics independently. |
| `pi-openui` / `pi-genui` spec compatibility adapters | Optional Rust parser/importer, read-only by default | Compatibility of old JSON/OpenUI Lang is useful but secondary to user capability parity; maintain one renderer only. |

**Migration policy:** Stage an isolated test with relevant extensions enabled and disabled. Document supported backend behavior and missing native TUI widgets. Do not uninstall, deregister or edit anything in the user's `~/.pi/agent` unless separately authorized. The frontend's default native visualization demo works even with both original UI extensions absent. Provide commands for rollback to ordinary `pi`.

**Theme distinction:** Pi's theme chosen in `settings.json` may be a bundled theme file, a JSON theme, or an extension programmatic theme. RPC does not expose the selected runtime TUI theme as an object. Implement import for documented file formats where accessible and safe, plus explicit theme config. Label unsupported programmatic Pi themes as requiring a frontend mapping; do not assert universal automatic inheritance.

### Configuration instructions

No changes to Pi `settings.json` are required merely to use the standalone RPC frontend. The old plan's `tuiMode=fullscreen` and `quietStartup=true` controlled Pi's interactive TUI and are **not requirements for this standalone Rust fullscreen UI**. Preserve them if the user wants native Pi to behave that way; do not set them automatically.

Propose a small optional user-owned config file, e.g. `~/.config/pi-visual/config.toml` (exact path follows platform conventions and should be documented), containing selected theme, renderer budgets, browser opt-in, safe file roots, graphics fallback, and keyboard preferences. Secrets must never live there. If absent, use safe defaults.

Kitty `background_opacity` remains an optional user-controlled `kitty.conf` setting; no remote control or automatic opacity changes. Transparent **canvas** + theme-tinted **cards** is the expected appearance. No misleading promise of alpha in every background cell.

---

## 13. Production verification matrix and acceptance standards

Automated tests are required from the first commit. Test at several layers without a model provider or personal credentials wherever possible.

### 13.1 Tests by layer

**RPC:** fake child process fixtures, malformed lines, oversized records, out-of-order IDs, delta reconstruction, cancellation/retry/settled ordering, session/branch replay, UI dialog request/response, multiple status keys, EOF/error cleanup, stdout/stderr separation, history cursor semantics.

**Terminal/editor:** snapshots at 40×12, 60×18, 80×24, 100×30, 120×40, 160×50; Unicode/emoji/CJK; cursor/wrapping; 100-line paste; undo/redo; image paste; `/`/`@` suggestions above; popup selection by keyboard/click/wheel; outside click; Escape; keyboard enhancement; modal restoration; scroll anchoring and jump-to-latest.

**Workflow:** live/finished message, nested/parallel events, tool success/failure/blocked, retry, cancellation, auto-compaction, bash/test/diff real vs synthetic fixtures, unknown tool fallback, streamed Markdown mid-fence and GFM table, no duplicate cards, exact resumption of transcript state.

**23 families:** a fixture and assertions for each table row §6; keyboard, mouse, local state, validation, tooltip, annotation, resize, narrow pan, contrast, accessible fallback, and zero-LM replay.

**Visualization:** property/fuzz tests for chart scales, null/NaN/Inf, graph routing, map coordinates, selection, annotation mapping, local formula grammar, schema migrations, deterministic simulation physics, energy drift and fixed timesteps.

**Graphics:** Kitty graphics probe, viewport placement, partial clip, multiple IDs, overlay z-order, image deletion, resize during animation, 30-minute memory/CPU stability, no ghost graphics on app exit. Verify fallback on unsupported/non-Kitty terminals and under tmux/zellij where feasible.

**Browser:** G3 interactive pendulum, two simultaneous local HTML widgets, actual pointer event forwarding and focus, wheel scroll, browser crash, sandbox denial of file/network/clipboard, dangerous navigation, resource leak, output sanitization, no unsandboxed fallback, no model calls per interaction.

**Security:** sandbox denies actual Pi tool writes, permission dialog works, no independent renderer-shell, browser content cannot read `~/.pi/agent`, no network in offline mode, Unicode/ANSI injection neutralized, symlink and oversized asset inputs denied, no secret disclosures, always recoverable cancel/exit.

**Reliability:** cold/warm startup, long sessions, 50+ concurrent/serialized cards, repeated session switches, terminal resize storms, theme changes, reconnect and Pi restarts, frame overload/backpressure, release tarball smoke test, panic recovery, no zombie Chromium/Pi child processes.

### 13.2 Quantitative targets and evidence

Define and publish a reproducible benchmark harness with at least:

- startup time and memory without Chromium; additional startup cost only on first browser preview;
- interactive input latency p50/p95 during one simulation and when two browser-backed widgets are visible;
- native and Chromium frame rate, dropped frames and terminal bytes/sec under real Kitty; no unsupported performance promises;
- incremental scroll and rendering cost with 100, 1,000 and 10,000 transcript entries, including Markdown and images, using viewport virtualization;
- browser resource limits and cleanup; native static fallback performance;
- Pi model call count, default tool manifest/context signature, and token delta exactly zero for UI-only actions;
- all 23 families with a completed **tested parity** matrix, including feature gaps and severity.

A passing unit test cannot substitute for real Kitty testing of graphics, mouse capture, security dialogs and IME. Mark CI-only approximations separately and include a verified manual-release checklist with actual screenshots/video where permitted.

### 13.3 Definition of DONE vs partial

A feature is **DONE** only when code exists, tests pass, real target-environment behavior was verified, documentation is current, and failures/fallbacks are truthful. Use statuses `missing`, `implemented-untested`, `tested-ci`, `tested-kitty`, `production-accepted`, `blocked`, not optimistic checkboxes.

**Production release gate:** G0–G8 pass, all 23 families have meaningful tested behavior, core simulation families work, native Markdown/composer/agent cards work, browser-backed interactive pendulum works safely in a scrollable chat, security/compatibility audits pass, and zero incremental default UI model usage is measured. If browser fails G3, release a clearly labeled *native-only beta* rather than claiming full scope. Report remaining work honestly.

---

## 14. Engineering phases, ordered for fastest validated progress

**Rule:** Implement working vertical slices in this order; do not begin by porting all 23 components or polishing the dashboard while the hardest technical uncertainty (inline Chromium + Kitty) remains unproven. Each phase ends in reviewed small commits, reproducible tests, a gate result and `docs/IMPLEMENTATION-STATUS.md` update. If a tool lacks network/Kitty/Chromium, test what is possible and record an explicit blocker; do not fake success.

| Stage | Actual implementation | Exit evidence / gate |
|---|---|---|
| **P00** | Inspect current official Pi RPC, Rust/Kitty/browser sources and the complete requirements in this document; ADRs, threat model, immutable requirements matrix, baseline versions | Verified source links, read-only audit, test plan and reproducible isolated workspace |
| **P01** | Create Cargo workspace, formatter/clippy/test/CI, ownership boundaries, minimal stdout/stderr log redaction | `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` |
| **P02** | Original `pi-rpc` typed transport, request correlation, event reducer, fake Pi fixtures | **G0** pass, no live user config writes |
| **P03** | Extension UI modal/permission paths and Pi sandbox negative tests | **G1** pass; fail closed |
| **P04** | Ratatui viewport, scroll virtualization, event loop, input routing, clean disposal; native static Markdown/tool card | Replayed chat works locally with no model and no Pi patches |
| **P05** | Kitty images + multiple scrollable image rectangles + precise clipping and cleanup | **G2** pass with real Kitty captures |
| **P06** | Minimal isolated Chromium/CDP sidecar, browser-to-Kitty frame protocol, mouse coordinate routing | Sidecar sandbox policy and frame transfer tests pass |
| **P07** | HTML/Canvas pendulum inline G3 with drag/sliders, scroll/focus/resize and malicious-content tests | **G3** pass; do not proceed with HTML-generalization on failure |
| **P08** | Native Rust pendulum fallback, safe spec engine, local controls and deterministic simulation clock | **G4** pass and zero model calls |
| **P09** | Production-grade composer, home/chat state, above-prompt `/`/`@` popup, Vi-mode option, minimal footer | **G5** pass with input integrity and source-backed stats |
| **P10** | Tokyo Night-inspired cards and the 18 self-contained visual states V01–V18; streaming Markdown and user-selectable themes | Real Kitty screenshots and state-by-state V01–V18 checklist, no fabricated data |
| **P11** | `piui/v1` schema validation, local state/expressions, GenUI adapters, OpenUI semantic matrix | **G6** initial vertical slice, no original UI extensions required |
| **P12** | All 23 component families plus tooltips, notes, choice staging, chart/diagram routing and galleries | 23/23 behavioral fixture tests and actual Kitty QA |
| **P13** | Remaining 6+ simulation families, native + optional browser rendering and local exports | Numerical reference tests, performance budget, no model updates |
| **P14** | Advanced inline HTML/JS scenarios (two widgets, SVG/D3/Canvas), browser sandbox hardening and static fallback | **G3** broader regression, denial/failure tests, offline operation |
| **P15** | Full Pi extension compatibility, theme import, packaged migration/rollback, security review | Pi sandbox/permissions proven, no config mutations |
| **P16** | Deterministic token/context audit, fuzz/property/golden/end-to-end/Kitty matrix and resource profiling | **G7** and **G8** pass with empirical evidence |
| **P17** | Release docs, reproducible binaries/packages, maintainer guide, third-party notice/license files, upgrade policy | Candidate production release with honest changelog and acceptance matrix |

**Critical path:** P02→P05→P07 is the feasibility path for inline HTML. A small safe native fallback may be released earlier, but not described as complete HTML parity. Large component work should wait until G3 risk is understood; independent tests/docs may proceed while browser feasibility is investigated.

### Execution checkpoints for Codex

At the end of each coding session:

1. State exact commit/working tree changes; list newly passing/failing/skipped tests with commands.
2. Update `docs/IMPLEMENTATION-STATUS.md`, `docs/feature-parity.md`, `docs/protocol-baseline.md` and any ADR changed by findings.
3. Give a minimal next implementation slice with clear acceptance and files likely to be edited. Avoid rewriting/repasting this whole prompt on every turn.
4. If lacking Kitty, Linux sandbox privileges or Chromium locally, distinguish compile/unit/PTY proof from target-terminal proof. Do **not** label feature production-ready.
5. Never ask to patch Pi. Never publish/push or change global config without explicit user authorization.

---

## 15. Files, documentation and long-term ownership deliverables

The repository must include actual working files, not just a readme pointing to TODOs.

| File/directory | Contract |
|---|---|
| `README.md` | Scope, official support targets, CLI startup, gallery, screenshots from real Kitty, warnings, quick rollback |
| `Cargo.toml` / `Cargo.lock` | Reproducible workspace/dependency graph and feature gating |
| `crates/pi-rpc` | Typed, tested official Pi RPC adapter with no Pitago dependency |
| `crates/tui-shell` | Complete OpenCode-style native frontend and functional popovers/composer |
| `crates/ui-schema`, `ui-runtime` | One versioned safe widget spec, stateful local controls, bounded expression parser |
| `crates/charts-graphs` | All 23 semantic families and graph/chart tests |
| `crates/simulation` | General physics + pendulum and six distinct core simulation demos |
| `crates/kitty-graphics` | Image/cell mapping, clipping, lifecycle and safe fallback |
| `crates/browser-bridge` | Optional isolated Chromium/HTML/JS integration, typed IPC, resource limits |
| `docs/architecture.md` | Component graph, process boundaries, modes, event/state ownership, decisions |
| `docs/protocol-baseline.md` | Tested Pi RPC commands/events, versions and incompatibilities |
| `docs/feature-parity.md` | 23/23 matrix with proof/test IDs, GenUI extras, known gaps |
| `docs/interaction-spec.md` | Keyboard focus, mouse coordinate model, accessibility, Vi mode, shortcuts |
| `docs/kitty-graphics.md` | Protocol capabilities, Unicode placeholders, z-order, scrolled viewport behavior |
| `docs/browser-security.md` | Chromium confinement, offline policy, interception, crash recovery and risk register |
| `docs/simulations.md` | Equations, numerical methods, unit conventions, correctness/energy drift tests |
| `docs/visual-design.md` | Semantic theme tokens and complete V01–V18 visual-state gallery, standalone text fixtures |
| `docs/token-budget.md` | Reproducible zero-incremental-token tests, opt-in authoring trade-offs |
| `docs/compatibility.md` | Pi 1.5.2/other tested versions, six extensions, Kitty, tmux/zellij limitations |
| `docs/install-upgrade-rollback.md` | Isolated install, normal executable startup, safe removal, no Pi patch/config changes |
| `docs/MAINTAINERS_GUIDE.md` | Ownership, version policy, upgrade watchlist, release and security response |
| `docs/MAINTENANCE_COSTS.md` | Grounded initial/recurring effort bands, assumptions, scope ceiling, major risks |
| `docs/IMPLEMENTATION-STATUS.md` | Explicit phases, pass/fail, commands, current blockers and next slice |
| `docs/ADRs/` | Key trade-offs and decision provenance |
| `examples/` | 23-component visual gallery, native simulations, user-supplied HTML/JS test fixtures |
| `tests/` and CI | Unit, property, fuzz, RPC fixture, permission, Kitty and browser e2e |
| `LICENSE`, `NOTICE`, `CHANGELOG.md` | Correct licensing/attribution and real release history |

All imports or copied code from other projects require actual evidence of rights and attribution. Any screenshots later provided by the user are optional layout examples, not code or redistributable assets by default. For world geography and fonts, verify redistribution rights and offline data provenance.

### Maintenance strategy: minimize third-party workflow ownership

- The **only required application-level *agent* protocol is Pi's public RPC contract**; Kitty is the terminal graphics/input protocol and Chromium/CDP is an optional rendering dependency. Every major library is a versioned Rust dependency with independent tests and a removal strategy; none of Pitago's project workflow controls yours.
- Maintain one `pi-rpc` compatibility suite against supported Pi releases, including the installed baseline and current latest tested release, run whenever Pi updates. Don't automatically update Pi or forcibly follow `main`.
- Track upstream Ratatui/Crossterm/ratatui-image/CDP changes with dependabot/renovate-style PRs **reviewed, not auto-merged**; pin lockfile and rebuild after tests. No `@latest` build command in production pipeline.
- Chromium sidecar is optional and higher maintenance/security cost: isolate its release cadence, version check, sandbox policy and emergency disable flag. Keep a minimal mode that starts without Chromium.
- Define realistic maintainer effort bands for a single Linux/Kitty-focused project and a larger cross-platform product; include initial RPC, complex graphics, browser security, regression QA, Rust toolchain/crate upgrades, OSS notices, CVE response, issue triage and docs. Do not invent precise weekly hours without assumptions.
- Support clearly documented feature removal and fallback when browser security/capability standards fail. Never promise indefinite support for specific GPU, terminal multiplexers or AI model behavior.

---

## 16. Installation and local testing instructions to produce in project

The coding agent must first inspect the real CLI and dependency versions, then provide **verified** commands in the project README. These are illustrative, not authorization to change existing user configuration:

```bash
# In the new, independently owned Rust repository:
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p pi-visual

# Offline/no-model gallery, run from Kitty:
./target/release/pi-visual --demo gallery
./target/release/pi-visual --demo pendulum

# Actual normal Pi RPC-backed client, without modifying Pi files:
./target/release/pi-visual

# View current limitations and capability probe, no Pi model calls:
./target/release/pi-visual doctor
```

Verify final executable names and args against **implemented CLI** before publication; don't create fake commands that only the README understands.

Use a temporary Pi config root (`PI_CODING_AGENT_DIR` only after confirming behavior for installed Pi) and throwaway project repo for integration tests. Do not copy the real auth file into test fixtures. External/model live tests are opt-in only; normal CI runs entirely offline/fake-provider. A browser e2e harness may use a locally installed Chromium and a controlled Kitty or virtual PTY display; never auto-download a browser without approval.

Do not ask the user to change `settings.json` merely because an old TypeScript plan did. The standalone Rust frontend uses Pi RPC; **`tuiMode` and `quietStartup` do not control its UI**. It may offer `--theme tokyonight` and `--browser-previews` only if actually implemented and opt-in behavior is made explicit.

---

## 17. Final reporting contract for the implementation agent

When each milestone or final release task finishes, answer with these headings and concrete evidence in this order:

1. **Repository status:** actual path, branch, commit SHA, clean/dirty tree, whether any remote was created (only if authorized). No fabricated published repo.
2. **What works now:** executable and user-visible features that were actually run; explicitly note native-only versus HTML-browser mode.
3. **Pi untouched proof:** no edits/patches to Pi executable/source or live `~/.pi/agent`, plus rollback verification and relevant file diffs.
4. **Critical gates G0–G8:** pass/fail/partial/not-tested, exact command and evidence for each, especially G3 with real Kitty interaction proof.
5. **OpenUI parity matrix:** all **23** families with implementation/test/Kitty status, cross-cutting notes/tooltips/keyboard/mouse/annotations and regression gaps.
6. **GenUI capabilities:** local reactive controls/calculator, schema/state, native double pendulum and the six other simulations; numerical correctness data.
7. **Inline HTML/JS:** which examples run, isolation/permissions, number of simultaneous widgets, graphics/cursor/focus limits, and browser-free fallback.
8. **Agent UI fidelity:** V01–V18 achieved/provisional/not yet, with real Kitty screenshots and honest reasons.
9. **Compatibility:** actual Pi/Kitty/crates/Chromium versions, six installed extension behavior in RPC and any Vi keymap limitations; security results.
10. **Tokens and resources:** reproducible model-call deltas (default UI target 0), presence/absence of opt-in tool schemas, CPU/RSS/frame/latency measurements, no fabricated throughput.
11. **Tests:** exact passed/failed/skipped counts, fuzz/property/e2e/manual target-environment evidence, remaining release blockers.
12. **Maintenance:** dependency/upgrade/CI/browser security burden, realistic effort bands and specific high-risk interfaces.
13. **Next smallest actionable slice:** precise files/tests to implement or fix next. Never claim full production readiness until all gates are met.

**Do not stop after generating another implementation plan.** Implement real vertical slices, compile and test the actual Rust code, and report limitations precisely. At the same time, do not mark difficult untested features complete simply because a static mockup resembles the required interface.

---

## Appendix A. Requirements traceability (self-contained)

Every row is binding and must be backed by actual implementation, tests and source evidence. This appendix expresses user-visible requirements directly; it is **not** a reference to another document.

| Domain | Exact required result | Evidence expected |
|---|---|---|
| Startup | Original ASCII PI logo, centered home composer, clean resource-free startup, home→chat transition | V02–V04 Kitty snapshots and transitions |
| Input | Real multiline/Vi editor, ghost text, `/` and `@` above prompt, focus/modal integrity | Input property tests, real keyboard/pointer/IME smoke tests |
| Footer | Left cwd, right cost then context; nothing else | Golden tests at 40/60/80/120/160 columns |
| Workflow | Tinted cards for all 18 states, truthful tool/permission/stream/summary information | V01/V07–V18 event-fixture tests |
| Markdown | Local GFM-like renderer; code, tables, safe links, raw-copy fidelity | Streaming and full-output snapshot tests, zero-model proof |
| OpenUI parity | All 23 families have real interactions, responsive layout, keyboard and mouse | `docs/feature-parity.md` with 23/23 passing behavioral evidence |
| GenUI parity | Reactive forms, sliders, state bindings, safe expressions, calculator, streaming spec and local actions | No-model interactive gallery and state serialization tests |
| Native simulations | Pendulums and six distinct core families, accurate local computations | Reference numerical tests, pause/step/reset, deterministic replay |
| Browser previews | User-approved real HTML/JS/Canvas in one *scrolling Kitty chat*; pointer/focus/resize | G3 live Kitty demo with two widgets and sandbox denials |
| Offline and zero tools | No extra provider-visible UI schema/calls in default mode; local-only render | G7 paired RPC/provider fixture and network-deny tests |
| Security | Pi sandbox preserved; independent browser isolation; no shell bypass | G1/G8 adversarial tests, no privilege changes |
| Ownership | No Pitago dependency, no Pi patch, independently pinned Rust crates | Lockfile, source audit, clean environment verification |
| Maintenance | Documented upgrade cadence, CI, licenses, crash cleanup and rollback | Release checklist and completed maintainer guide |

### Product boundaries — intentional differences from prior terminal extensions

- Render OpenUI and GenUI **capabilities**, not their original TypeScript widget instances. A single Rust `piui/v1` runtime owns drawing and interaction. Optional importers may decode compatible legacy descriptions into the local schema but cannot assume full semantic equivalence without fixtures.
- The standalone frontend uses Pi's **RPC interface**, not Pi's internal TUI extension objects. Pi's native UI `custom()`/header/footer/editor/theme APIs are unavailable or reduced in RPC; implement independent Rust equivalents without patches.
- One model-facing `render_ui`/`render_widget` tool is **not** installed by default. A user-authorized future authoring adapter may be tested separately with transparent nonzero token overhead, versioned schema and clear removal path.
- Real HTML/JavaScript receives a browser sandbox, explicit consent and native fallback. Web-only proprietary message components/bindings are *not* guaranteed to execute from raw copied snippets; only documented portable HTML/CSS/JS or explicitly implemented adapters are supported.
- The frontend and optional Chromium are independently responsible for filesystem/network boundaries; Pi's tool sandbox is not a blanket sandbox for all renderer subprocesses.

---

## Appendix B. Research links (verify versions before coding)

- Pi CLI integration and RPC: https://docs.pi.dev/latest/cli-integration
- Pi RPC docs pinned to input baseline: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc.md
- RPC commands: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc-commands.md
- RPC event stream: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/json.md
- RPC extension UI limitations and dialogs: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc-extension-ui.md
- Pi-genui capability reference: https://github.com/chrischriscris/pi-genui
- Pi-openui component gallery: https://github.com/newtonapple/pi-openui/blob/main/docs/components/README.md
- Pi-openui interactions: https://github.com/newtonapple/pi-openui/blob/main/docs/interaction.md
- Pitago reference only: https://github.com/cavaldos/pitago
- Ratatui: https://ratatui.rs/
- Crossterm: https://docs.rs/crossterm/latest/crossterm/
- ratatui-image: https://github.com/ratatui/ratatui-image
- Kitty graphics and Unicode placeholder: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Ratatui editor candidate: https://github.com/ratatui/ratatui-textarea
- Rust Markdown candidate: https://docs.rs/tui-markdown/latest/tui_markdown/
- Rust alternate Markdown candidate: https://docs.rs/ratatui-markdown/latest/ratatui_markdown/
- Rust Chromium/CDP candidate: https://docs.rs/headless_chrome/latest/headless_chrome/
- Rust Kitty charts concept reference: https://docs.rs/kittychart-ratatui/latest/kittychart_ratatui/
- OpenCode source: https://github.com/anomalyco/opencode
- Tokyo Night: https://github.com/folke/tokyonight.nvim

## Final acceptance sentence

**The user can launch a Rust-owned, visually polished OpenCode-style coding agent frontend in Kitty, using the existing completely unmodified Pi backend, with functional `/` and `@` popups, truthful agent cards, local Markdown, full 23-family OpenUI-class widgets, GenUI-class reactive forms/charts/animations, correct interactive pendulums and other simulations, and explicitly approved isolated HTML/JavaScript/Canvas widgets rendered inline within a scrolling chat—all with zero additional LLM calls/tokens for presentation or local interaction, verifiable security and transparent maintenance/rollback.**
