# OpenCode-style UI for Pi — revised production implementation plan

**Document status:** Research-backed engineering plan, **not a completed extension**.  
**Target:** Pi Coding Agent **1.5.2**, `@earendil-works/pi-coding-agent`, Kitty on Linux, default agent directory `~/.pi/agent`.  
**Deliverable after implementation:** Exactly **one runtime file**, `~/.pi/agent/extensions/opencode-ui.ts`, with no `pi-fast-tui` dependency.  
**Other installed extensions:** `pi-sandbox`, `pi-cache-hit-monitor`, `pi-openui`, `pi-permission-system`, `pi-lens`, `pi-feature-dev`.  
**Last plan revision:** 15 November 2026.  
**Design priorities:** Preserved Pi behavior, transparent surfaces, live inherited themes, exact footer hierarchy, **autocomplete above the composer**, token-free UI, and a lightweight OpenCode-inspired agentic experience.

> **Evidence convention:** “Verified” refers to code or documentation examined at the linked Pi 1.5.2 / OpenCode source. “Proposed” is design work still requiring implementation and Kitty acceptance testing. The embedded pictures below are **rendered design mockups**, **not screenshots of a running extension**. They are composited over sample Kitty backgrounds solely to make the transparent UI legible. Corresponding **alpha-transparent PNG overlays** are supplied in `assets/`.

---

## 1. Superseding design decisions

This revision **replaces** the prior implementation plan wherever it disagrees:

| Area | Previous plan | **New specification** |
|---|---|---|
| Footer left | Telemetry or status | **Current working directory** (`ctx.cwd`) |
| Footer right | Model, effort, context, cost | **Estimated/known session cost, then context used/window**, right-aligned |
| Footer omitted | — | **No LSP status, no sandbox status, no model, no thinking effort** |
| Composer | Shaded OpenCode-like area | **OpenCode-style geometry, rail, border, placeholder, but transparent interior** |
| Startup | Pi logo and adaptive composer | Same, on a clean transparent home screen |
| `/` autocomplete | Native behavior | **Keyboard/mouse-operable popover immediately ABOVE composer** |
| `@` autocomplete | Native behavior | **Same above-composer popover**; file search remains native |
| Agent execution visuals | Mostly native | **Observed tool steps, progress, outcome, grouped tool activity and streamed response** |
| Themes | Custom palette resembling screenshot | **Every owned element inherits the theme currently active in Pi**, including third-party/package themes |
| Color backgrounds | Dark slate painted behind UI | **No painted background for dashboard, composer, suggestions, activity strip, or footer** |
| Deliverable | Single extension | Remains a single **`opencode-ui.ts`**; tests/mockups separate, no runtime dependencies |

**Non-negotiable:** Visual changes may not bypass `pi-permission-system`, `pi-sandbox`, native tool execution, LSP, file completions, or interactive `pi-openui` components. Removing a status from the footer does **not** disable its extension or suppress its warnings/approval dialogs.

---

## 2. Rendered appearance: approved target states

### 2.1 Fresh session — minimal transparent dashboard

![Rendered mockup of the initial Pi dashboard: centered Pi ASCII logo, upper-middle transparent composer, minimal bottom footer.](assets/01-home.png)

- Exactly one ASCII-art **Pi** icon. No Pi version, CLI help, `[Context]`, `[Skills]`, extension list, or onboarding prose in an ordinary healthy startup.
- Composer starts near the upper-middle: around 45–55% of available terminal height, responsive to row count.
- No full-width solid background behind the logo, composer, or footer. The accent rail and fine rules provide separation.
- Composer metadata **retains** current mode, selected model and thinking effort, as requested. Those **do not** reappear in the footer.
- Bottom left: current directory. Bottom right: cost and context usage. In empty sessions, show truthful zero/unavailable states, not fictitious billing.

**True transparent asset:** [`assets/01-home-transparent.png`](assets/01-home-transparent.png). If Kitty is transparent, a desktop wallpaper or other windows can be visible behind the unpainted cells.

### 2.2 Active agentic workflow — transcript + bottom-docked composer

![Rendered mockup of active agentic run, with observed tool steps above an anchored transparent composer and a two-sided bottom footer.](assets/02-agent-workflow.png)

- Upon the first committed conversation/workflow action, hide the home logo and remove the lower spacer; composer remains bottom-docked for the active session.
- Conversation and tool details scroll in Pi's native fullscreen transcript viewport.
- A lightweight, **read-only activity view** can show current tool execution and several recent steps while work is underway; no fabricated “plan” or subagent status.
- Keep actual permission dialogs and error output visible, even though they may interrupt the clean appearance.
- The running display must never introduce extra model calls or context messages.

**True transparent asset:** [`assets/02-agent-workflow-transparent.png`](assets/02-agent-workflow-transparent.png).

### 2.3 `/` commands — popover ABOVE the prompt

![Rendered mockup of slash-command suggestions appearing in a themed transparent popup immediately above the prompt.](assets/03-slash-popup.png)

- Typing `/` opens the native command-completion results **directly above**, not below, the editor.
- Suggested rows are selectable by arrow keys, Tab/Enter, and pointer when Pi's original completion supports that action.
- Use a thin top/bottom outline and the theme's accent for the highlighted row; no opaque selection rectangle.
- Respect Pi's native command handling, including command sources from other installed extensions.

**True transparent asset:** [`assets/03-slash-popup-transparent.png`](assets/03-slash-popup-transparent.png).

### 2.4 `@` project files — same positioned popup

![Rendered mockup of file-reference suggestions above the prompt, with transparent background and themed active row.](assets/04-at-popup.png)

- Typing `@` and a partial path opens native file-reference suggestions in the **same above-composer visual system**.
- Retain fuzzy matching, file filtering, completion insertion, invalid-file/no-match states and terminal-input semantics.
- Do not implement a new filesystem crawler or duplicate Pi's existing file-reference provider.

**True transparent asset:** [`assets/04-at-popup-transparent.png`](assets/04-at-popup-transparent.png).

### 2.5 A different active theme must update everything

![Rendered mockup of the same transparent dashboard under a light terminal/theme palette.](assets/05-light-theme.png)

All text, rails, rules, prompt metadata, placeholder, selection, activity and footer colors must be derived from the **active Pi theme roles**; this light mockup is a proof-of-design, not a proposed forced light theme.

**True transparent asset:** [`assets/05-light-theme-transparent.png`](assets/05-light-theme-transparent.png).

**Mockup caveat:** Exact pixel matching to OpenCode is not technically guaranteed: OpenCode uses OpenTUI's dedicated home/session routes, while Pi 1.5.2 uses a transcript + fixed input dock. The target is a *visually close, interaction-safe approximation* using public Pi extension interfaces.

---

## 3. Changes required in `settings.json`

Edit your existing `~/.pi/agent/settings.json` and **merge**, never replace, these two keys:

```json
{
  "tuiMode": "fullscreen",
  "quietStartup": true
}
```

| Setting | Required? | Why |
|---|---|---|
| `tuiMode: "fullscreen"` | **Yes** | Pi's independent scrollable transcript and fixed composer/footer are required for adaptive positioning. |
| `quietStartup: true` | **Yes** | Hides native Pi welcome/header and normal context/skills resource lists. Does **not** stop those resources loading into the model. |
| `theme` | **No** | Keep whichever built-in, `system`, custom, project, or extension-package theme you already choose in `/settings`. |
| `autocompleteMaxVisible` | Optional | Limit popup rows to suit screen size, e.g. `6`. The extension additionally bounds rows dynamically. |
| `editorPaddingX` | Usually leave unchanged | Respect Pi's configured text input padding; do not force 0 globally. |
| `showHardwareCursor` | No change by default | Do not disrupt the user's cursor/IME preference. |
| Other installed extension settings | **No changes** | Do not remove `packages`, paths, permissions, sandbox config, lens settings, or MCP settings. |

**If the Pi settings file already contains a `theme` key, leave it alone.** The implementation must not force `theme: "system"`, `"dark"`, or any private theme. Package-provided custom themes continue to be selected the normal Pi way. Project-local settings may override global settings, so validate the effective mode in the active project.

### Kitty-specific transparency (not Pi settings)

Pi's terminal output cannot make an opaque Kitty window transparent by itself. To allow the desktop behind the terminal to show through, optionally add this to your Kitty configuration (commonly `~/.config/kitty/kitty.conf`):

```conf
# Optional: only if you actually want desktop-through transparency
background_opacity 0.88
```

- `1.0` is opaque; numbers below `1.0` are transparent if the operating system/compositor supports it. Choose your own value.
- `dynamic_background_opacity yes` is **optional**, required only if you want to adjust opacity dynamically; it adds overhead.
- Our UI component rendering must emit **no explicit terminal background-color escape** (`SGR 48;...`, `selectedBg`, etc.) for its own surfaces. This is independent of Kitty's window opacity.
- Pi's **native transcript** and **other extensions** may still color their own cells. This single UI extension cannot guarantee arbitrary third-party widgets are transparent without altering or replacing their renderers/themes. Where possible, choose a theme whose background roles use terminal defaults; never silently rewrite another theme file.
- Use a sample background only for visual previews; **do not encode a dark rectangle as the shipped UI's background**.

Sources: [Pi 1.5.2 themes](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/themes.md), [Kitty opacity documentation](https://sw.kovidgoyal.net/kitty/conf/#opt-kitty.background_opacity).

---

## 4. Verified upstream architecture and extension interfaces

### Pi 1.5.2

- Fullscreen viewport: [`chat-viewport.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/chat-viewport.ts) uses a vertically growing `ScrollView` transcript and a fixed dock containing pending messages → status → **above-editor widgets** → editor → **below-editor widgets** → footer.
- Startup: [`interactive-mode.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/interactive-mode.ts) creates built-in header and loaded-resources containers independently; quiet startup is essential. `setHeader` alone cannot hide loaded resource lists.
- Editor: [`editor.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/editor.ts) retains built-in clipboard, cursor, wrapping, history, paste markers, autocomplete and mouse handling.
- Extension registration: [`extensions.md`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/extensions.md) provides `ctx.ui.setHeader()`, `setEditorComponent()`, `setWidget()`, `setFooter()`, `session_start`, `session_shutdown`, `session_tree`, `input`, `before_agent_start`, `tool_execution_*`, and agent lifecycle events.
- Footer data: [`footer-data-provider.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/core/footer-data-provider.ts) supports Git branch/status maps; the new minimalist footer deliberately does **not** surface extension status values.
- Example implementations: [`border-status-editor.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/border-status-editor.ts) and [`custom-footer.ts`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/custom-footer.ts).

### OpenCode reference

- [`Home`](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/home.tsx) centers a logo and prompt between flexible vertical spacers. This is the *initial-view behavior* to approximate, not an API Pi implements natively.
- [`Prompt`](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/component/prompt/index.tsx) owns a multiline editor, placeholder, selection, model/effort metadata and completion UI.
- [`Session`](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/session/index.tsx) is the inspiration for chat + tool progress + status. Do not copy its source verbatim or assume Pi uses the same runtime data structures.

### Important feasibility distinction: suggestions above the editor

**Verified:** Pi's native `Editor.render()` currently appends its autocomplete `SelectList` **after** the bottom editor border. Its `handleMouse()` maps clicks to those lower rows. There is no documented single setting or supported `autocompletePlacement: "above"` flag in the inspected Pi 1.5.2 API.

**Proposed first implementation:** a `CustomEditor` subclass that preserves Pi's original completion provider, `super.handleInput`, `super.render` and text state, but **reorders the presentation of the generated completion rows above the editor**, with corresponding pointer-coordinate remapping. It must not mutate private editor properties via `as any` or duplicate command/file searches.

**Mandatory feasibility gate:** prove this is stable for `/` and `@`, long or wrapped input, no-matches, selected-row updates, scrollable suggestions, and mouse clicks before claiming production readiness. If it cannot be implemented safely through the public subclass render/mouse methods, stop and prepare a narrowly scoped **upstream Pi `Editor` API proposal** exposing completion placement, rather than shipping a brittle monkeypatch or lying about feature parity. The one-file extension requirement remains; any upstream API dependency must be called out before shipping.

---

## 5. Single-file module architecture

The final code must be one discoverable TypeScript file (no extra `.ts` runtime dependencies):

```text
~/.pi/agent/extensions/opencode-ui.ts
├─ 01  Imports / version guard / feature flags
├─ 02  Read-only theme selector + semantic token mapping
├─ 03  ANSI/Unicode width, truncation and wrap helpers
├─ 04  Session state + first-action transition state machine
├─ 05  Viewport geometry and low-height fallbacks
├─ 06  Transparent, centered Pi ASCII logo/header
├─ 07  Adaptive spacer below the editor
├─ 08  OpenCode-style CustomEditor subclass
│      ├─ transparent composer/rail/borders
│      ├─ semantic mode/model/effort row
│      ├─ local ghost placeholder
│      └─ native completion popover relocation
├─ 09  Above-editor popup geometry/mouse-coordinate adapter
├─ 10  Read-only agent + tool activity accumulator
├─ 11  Optional compact agent-activity widget
├─ 12  Usage and context selectors (no invented values)
├─ 13  Two-sided, transparent directory/cost/context footer
├─ 14  Pi lifecycle wiring + failure handling + cleanup
└─ 15  Internal configuration and one public default export
```

Use explicit internal constants such as `HOME_EDITOR_FRACTION`, `HOME_LOGO_FRACTION`, `MAX_SUGGESTIONS`, `MAX_ACTIVITY_ROWS`, and `HOME_MIN_ROWS`. No new package dependencies, shell subprocesses, background networking, model calls, tools, MCP registrations, or dynamic code loading.

**Do not migrate the old `exit.ts` alias without separate permission.** It is unrelated to the UI, and the extension should not take ownership of built-in or third-party commands. If the old standalone `adaptive-prompt.ts` is present, disable it before enabling the new unified extension to prevent double spacing.

---

## 6. Theme inheritance and transparent rendering contract

**Fundamental rule:** The extension is a **consumer of the selected Pi theme**, not a competing theme manager.

### Semantic roles

| UI element | Preferred active Pi role | No-background rule |
|---|---|---|
| Pi ASCII logo | `accent`, `borderAccent` | Foreground only |
| Composer left rail | `accent` | Foreground glyph/line only |
| Composer border/hairlines | `border`, `borderMuted` | No fill |
| Editor primary input | `text` | Terminal default background |
| Ghost placeholder | `dim` / `muted` | Terminal default background |
| Composer metadata mode | `accent` | Foreground only |
| Composer model / thinking | `muted` / `thinking*` | Foreground only |
| Popup outline | `border` | No fill |
| Popup selected row | `accent` plus left marker / weight | **Never** require `selectedBg` fill |
| Popup inactive rows | `text`, `muted` | No fill |
| Tool success / failure | `success`, `error` | No fill |
| Tool executing | `accent` | No fill |
| Footer directory | `muted` | No fill |
| Footer cost / context | `text` / `muted` | No fill |

Do not embed fixed hex palettes in the runtime (the pictured previews happen to use dark and light sample palettes). During `render()`, look up **the current** `ctx.ui.theme` or the current theme reference supplied by Pi; avoid capturing preformatted ANSI strings that become stale after theme switching. Use `visibleWidth()`/`truncateToWidth()`-style helpers and Unicode-aware clipping so ANSI escape codes and wide glyphs do not corrupt alignment.

### Theme-source compatibility

Test active themes from: Pi's `system`, `dark`, and `light`; global `~/.pi/agent/themes/*.json`; trusted `.pi/themes`; package/extension-provided themes; and themes selected at runtime via `/settings` or CLI. The extension must not set a theme or redefine the `theme` setting on startup. Re-render on existing Pi theme change signals and normal invalidation; no polling or model interaction.

### Scope of the transparency promise

Our **own five surfaces** (home, composer, popup, activity, footer) emit no opaque background cells. Pi's core transcript, selected third-party panels, and Kitty's own window background are separately owned. Preserve readability when the user chooses a theme with low foreground contrast or a translucent desktop wallpaper.

---

## 7. Dashboard / adaptive positioning specification

The empty-session experience is a presentation phase, **not a model message**. Use a state machine:

```text
STARTING → HOME (empty history)
         → CHAT (existing history)
HOME --first committed user/agent/shell work--> CHAT
CHAT --/new or /resume(empty)---------------> HOME
CHAT --/tree to empty branch----------------> HOME
HOME/CHAT --/reload-------------------------> derive from active branch
HOME/CHAT --shutdown------------------------> cleanup
```

- Detect meaningful conversation using `ctx.sessionManager.getBranch()` plus state of an in-flight first submission. Ignore bookkeeping-only entries, theme changes, `/settings` opening, a bare `/` or `@` edit, and non-agent UI commands.
- Early transition hooks: `input` (typed user prompt), `before_agent_start`, and `agent_start` are complementary; `input` does **not** fire for extension commands, so `before_agent_start` covers custom agent initiation. `user_bash` handles a shell action that produces transcript content. Do not dismiss HOME merely because autocomplete is shown.
- Show Pi ASCII art centered horizontally and placed around one quarter of terminal height. Calculate leading header whitespace, editor row, and below-editor spacer from current terminal **rows**. Clamp all to viewport and footer height. Do not hardcode row coordinates.
- The composer begins at about 48% of the terminal height on a tall new screen and ends at the normal bottom dock when work begins. A single render changes phase cleanly. **No animation** in v1: per-frame movement risks input focus and cursor desynchronization.
- `setHeader()` is used only for startup logo and its spacing, not for normal user-visible instruction lists. A custom component may render more rows than array widgets' 10-line cap, but it must still obey available terminal height.
- On narrow/short screens (e.g. 60×18), prioritize usable editor and footer, hide/miniaturize the logo, reduce spacer to zero, and shorten metadata.
- On session resume, branch switch and `session_tree`, immediately recompute from **actual history**; clear stale popup/activity state so one session's suggestions never appear in another.
- Critical warning/permission UI remains untouched. `/reload` and CLI initial-prompt behavior must not produce a stuck dashboard.

**Geometry targets, not rigid equations:**

```text
termRows              = tui.terminal.rows
footerRows            = measured custom footer height
editorRows            = measured active editor height
homeTargetEditorTop    = clamp(round(termRows * 0.52), safeMin, safeMax)
belowEditorHomeSpacer = clamp(termRows - footerRows - editorRows - homeTargetEditorTop, 0, available)
logoTop               = clamp(round(termRows * 0.20), 0, visibleTranscriptRows - logoRows)
activeSpacer          = 0
```

The actual code must measure its dynamic popup/editor heights rather than assuming three fixed editor lines.

---

## 8. Transparent OpenCode-style composer and ghost text

**Implementation base:** Subclass Pi's supported `CustomEditor`; never replace it with an independent fake input. Preserve its `onSubmit`, `onChange`, keybindings, history, autocomplete provider, native paste markers, image paste, multiline editor, undo, selection, hardware-cursor marker, and focus.

### Visual contract

- Thin accent rail at left; understated top/bottom rules in themed border color; no background fill.
- Default width responds to Pi's available terminal columns. Where public APIs permit it, use balanced horizontal margins; otherwise avoid unsupported core layout patches and document achievable approximation.
- Inside composer, the first row is editable text. When empty and idle, render the screenshot-like **visual placeholder** `Ask anything...  "Fix a TODO in the codebase"` (example wording); never insert this text into editor state or the LLM prompt.
- Placeholder disappears when content exists, including an image/paste marker or in-progress composition; returns on clearing input.
- Below input: only current interaction mode (e.g. `Chat`), model label, thinking effort. These are **inside the composer**, not in the footer. Use actual `ctx.model` and Pi thinking level; no hardcoded `GPT-6` or `high` in runtime.
- Native layout engine continues to handle text wrapping, variable editor height, completion and text cursor. Rendering must not alter byte count or submit behavior.
- The composer remains keyboard accessible and contrast-aware in dark, light, and custom themes.

**Ghost text here means placeholder**, not per-keystroke predictive code/text completion. Adding model-powered inline suggestions would consume tokens and is explicitly **out of scope**.

---

## 9. `/` and `@` popups ABOVE the composer — implementation gate

The screenshots make this a **required target behavior**, not a cosmetic preference.

### Functional requirements

1. `/` shows command suggestions from built-in commands **and** commands registered by extensions; preserve Pi's ordering, descriptions, filtering, and insertion/submit semantics.
2. `@` shows native file/path references with fuzzy ranking; no new disk scans; avoid exposing unexpected files outside Pi's existing provider behavior.
3. Popup is immediately **above** the visible composer in both HOME and CHAT phases; it never appears below the composer or behind the footer.
4. Popup is transparent and inherits active theme roles; selected row indicated by accent rail/text, not an opaque rectangle.
5. All keyboard controls (up/down, page keys, Tab, Enter, Esc, backspace, text edit) must retain upstream behavior. File completion must never inadvertently submit the prompt.
6. Mouse wheel/click coordinates are remapped correctly to Pi's native `SelectList` after rows are visually repositioned.
7. Popup height is bounded by free screen rows; when insufficient, show fewer suggestions with Pi's native scrolling behavior; do not overlap permission dialogs.
8. No flash/jump/focus-loss when suggestions open, narrow down, disappear, or when a prompt becomes multiline.
9. Works with terminal and active themes switched live, including long file paths and wide Unicode characters.
10. Must not steal keyboard handling from `pi-openui` or approval prompts.

### Recommended approach and fallback

**A. Preferred extension-only feasibility spike:** subclass editor; call `super.render()` exactly once per frame; identify the native autocomplete lines *without inspecting/mutating private data*; arrange them before the composer border, reapply theme-only decoration, and translate mouse coordinates in overridden `handleMouse()`. Keep original `super.handleInput()` as the source of truth. Validate Pi 1.5.2's native line ordering and overflow treatment with tests. If extraction cannot be made unambiguous for scrolled input, fail the gate.

**B. Consider a supported overlay/widget composition:** only if it preserves focus, suggestion state, and exact placement across TUI resize. A second autocomplete provider or independently queried file list is **not** acceptable because it loses Pi source-of-truth behavior.

**C. If A/B fail:** define a small upstream Pi TUI API enhancement (for example `autocompletePlacement: "above" | "below"` / render slot) and defer declaring this extension production-ready until that API exists. **Do not ship** `as any` private-field access, prototype monkeypatching, swallowed input, or a popup that looks right but breaks mouse/IME/Tab.

**Proof of feasibility must precede the rest of final implementation.** This is the highest technical risk, despite how simple the mockup looks.

---

## 10. OpenCode-inspired **agentic workflow** presentation

This is a **read-only UI layer over Pi's existing agent loop**, not a replacement orchestrator, autonomous planner, or permission engine. The goal is to make progress visually coherent like OpenCode while preserving Pi's model/tool messages and observability.

### Event mapping (verified Pi extension event family)

| Agent/workflow event | UI behavior | Rule |
|---|---|---|
| `before_agent_start`, `agent_start` | Hide HOME, show running state | Does not inject any prompt text |
| `message_start` / `message_update` / `message_end` | Let Pi stream response normally; optionally show a small active label | Never duplicate entire LLM output in a widget |
| `tool_execution_start` | Add pending tool activity row | Observe, do not modify args or approve calls |
| `tool_execution_update` | Refresh current step when meaningful | Throttle/batch updates; do not render every token |
| `tool_execution_end` | Mark succeeded, failed or blocked **only where result proves that state**; record duration if known | Handle out-of-order results/parallel calls by `toolCallId` |
| `turn_start` / `turn_end` | Track turn boundaries | Do not imply one turn equals one completed task |
| `agent_settled` | Hide activity widget or show a short final state; leave transcript intact | Distinguish aborted vs completed when event provides it |
| `session_start` / `session_shutdown` / `session_tree` | Rebuild/dispose active view | No stale tool rows on other sessions |

Read source: [Pi tool execution events](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/extensions.md) (search `tool_execution_start / tool_execution_update / tool_execution_end`).

### UI behavior

- **During a run:** one compact, transparent activity group above the composer, e.g. `Reading src/parser.ts`, `Searching tests`, `Running tests`, with themed status symbols and durations. Limit to last 3–5 steps; show a concise overflow indicator. Never move the bottom composer merely because progress updates arrive.
- **While tools run concurrently:** keep rows keyed by `toolCallId`, count active tools, report successes/failures only once per call, and avoid implying sequential execution.
- **When the agent returns a response:** the full answer stays in the original Pi transcript; the activity group disappears or collapses. Preserve `/tree`, tool expansion, and scroll behavior.
- **When approval is required:** let `pi-permission-system` and Pi's own dialogs take focus; pause decorative UI effects. Do not display a deceptive green success status until the tool has actually completed.
- **On errors, cancellation, or provider failure:** show truthful state using result/status, do not hide messages or simulate completion.
- **On agents/subagents:** show subagent hierarchy or role chips only if the actual installed orchestrator emits reliable structured events. The six stated packages do not, by themselves, justify inventing 4-role coordination or progress percentages.
- **No native renderer override initially:** Pi's transcript remains the source of truth for tool results and diffs. Avoid re-registering `read`, `bash`, `edit`, or other tool definitions just for styling, because those tools may be wrapped by sandbox/permission extensions. More ambitious OpenCode-style collapsible tool result cards are a later separate compatibility project, not a silent requirement of v1.

### Proposed agent display states

```text
IDLE
  └─ first submission → STARTING
STARTING
  └─ provider/agent begins → THINKING_OR_RESPONDING
THINKING_OR_RESPONDING
  ├─ tool started → TOOLS_ACTIVE (1..n simultaneous)
  ├─ permission requested → WAITING_FOR_USER (host-owned dialog)
  └─ final reply → SETTLED
TOOLS_ACTIVE
  ├─ update → TOOLS_ACTIVE
  ├─ all tools ended → THINKING_OR_RESPONDING
  └─ interruption/error → ABORTED/FAILED as known
SETTLED
  └─ new user prompt → STARTING
```

Use only state evidenced by Pi callbacks; avoid pretending to know private model reasoning or ETA. The workflow UI must never add instructions, tools or extra tokens to LLM context.

---

## 11. Footer specification — **exact requested layout**

**Single minimal row, bottom-anchored** (in normal terminal widths):

```text
~/Projects/my-project                         $0.014 est  ·  Context 18% / 272k
```

The directory is **left-aligned** and cost/context is **right-aligned**, with natural space between. Do not show model, thinking effort, LSP status, sandbox status, git branch, extension names, command hints, or a progress spinner in the footer.

**Data sources:**

- Directory: `ctx.cwd` at the current session; shorten the home prefix to `~` only at a path boundary; do not infer cwd from Git or prompt text.
- Context: `ctx.getContextUsage()` where available; display measured usage and model window. If a percentage or denominator is unavailable, show an honest `Context —` or available partial value.
- Cost: total **reported estimated** spend from the session branch's assistant usage and applicable usage entries where documented, without double counting; if current provider does not report dollars, use `Cost —`, not `$0.00` unless measured zero. Subscription-labeled traffic requires a caveat; do not call it actual user billing.
- No need to fetch any external data. Update on turn settlement, model changes, usage-relevant events and resize; no periodic polling.
- Full accuracy outweighs decorative precision; avoid false dollar digits.

**Narrow terminal behavior:** show `~/…/project` or end-preserving directory ellipsis left and compact right e.g. `$0.01 est · 18%/272k`; when even that cannot fit, prioritize preserving `Context`, show truthful abbreviated cost, and never overlap/crop visible graphemes. Use Unicode display width and theme-dependent ANSI spans for exact alignment.

**Important safety nuance:** Removing sandbox/LSP from *this footer* does not change `pi-sandbox`, `pi-lens`, or permission behavior. Their explicit notifications, diagnostic widgets, or permission prompts must still appear when triggered. Keep their extension-published status data intact in Pi even though the custom footer chooses not to render it.

---

## 12. Installed-extension coexistence matrix

| Existing | UI interaction | Required integration |
|---|---|---|
| `pi-sandbox` | Publishes status, manages sandbox/tool restrictions, may prompt | **Do not render routine sandbox status in footer**; preserve real warning/confirm UI and security execution paths |
| `pi-cache-hit-monitor` | May own an **above-editor** cache widget | Preserve the widget; activity and popup layout must avoid collision and clamp heights |
| `pi-openui` | Owns interactive `render_widget` tool/own hint widget and input handling | Never swallow its shortcuts, replace its components, or change tool definition |
| `pi-permission-system` | Policy decisions/dialogs | Preserve every allow/ask/deny path; no cosmetic suppression of approvals |
| `pi-lens` | LSP status and language tools | **Do not show routine LSP status in footer**, but keep its diagnostics/notifications intact |
| `pi-feature-dev` | Skill resources and workflow instructions | Keep resources loaded and usable; `quietStartup` affects only visible startup resource list |

Other custom editor or footer extensions, if added later, would compete for `setEditorComponent()` or `setFooter()` ownership. Detect and report that conflict; do not repeatedly reinstall to win a race. `pi-fast-tui` must remain uninstalled for the requested scope.

---

## 13. Reliability, performance, token, and security guarantees

**Runtime rules**

1. One public default extension export, one unique widget namespace (`opencode-ui:*`), no external runtime files. Use `session_start` for installation and `session_shutdown` for lifecycle cleanup; `session_tree` for branch navigation. There is **no** documented `session_switch` event in Pi 1.5.2.
2. No change to active tool list, MCP server list, `AGENTS.md`, policy hooks, `systemPrompt`, request payload or model; do not re-register sensitive built-in tools.
3. No model-facing tool or injected prompt. **Incremental model tokens used by the UI implementation: zero**, even though already-installed extensions retain their own token costs.
4. All layout rendering deterministic, bounded by terminal rows and columns; no new per-keystroke synchronous filesystem scans, API calls, tokenization, or heavy timers.
5. Avoid background processes/intervals. If an animation is eventually added, it must be opt-in, disposable, and tested for render/cursor safety; phase transition has no animation initially.
6. No global TUI monkeypatch, private-property introspection, terminal escape injection from untrusted tool output, or forced theme change.
7. Use safe truncation/ANSI handling for directory strings and model names. Sanitize control characters from filenames/tool metadata in activity rows and popups to prevent terminal escape abuse.
8. Fail open visually, **not** for permissions: on cosmetic renderer failure, restore Pi's editor/footer and leave security hooks untouched. Log concise diagnostics without flooding the transcript.
9. Preserve focus, image paste, bracketed paste, IME, hardware cursor, long multiline input, command history, copy/paste, and tool expansion.
10. Reconfirm compatibility after every Pi minor update. Explicitly target Pi 1.5.2 in a header comment; avoid silently claiming compatibility with later versions.

---

## 14. Phased engineering execution plan

| Phase | Task | Exit criteria |
|---|---|---|
| **P0 Baseline** | Capture versions, installed extensions, existing settings, active theme, Kitty dimensions; back up `settings.json` and old `adaptive-prompt.ts` | Working Pi reproducible, rollback available |
| **P1 Popup feasibility spike** | Prove `/` and `@` completion can move above with native editor keyboard + mouse behavior, **without private API** | **Hard gate**: all autocomplete focus, row selection, Tab, Enter, Esc, resize, mouse tests pass; otherwise stop for upstream proposal |
| **P2 Clean startup** | Merge `tuiMode`, `quietStartup`; render transparent ASCII Pi header and hide ordinary startup clutter | Empty dashboard shows no built-in version/context/skills listing; real diagnostics remain accessible |
| **P3 Adaptive geometry** | Integrate old spacer functionality into home/chat state machine, no separate extension | Home near middle, chat at bottom, stable on `new/resume/tree/reload` and resize |
| **P4 Composer** | Extend `CustomEditor`, add transparent accent rail/rules, model/effort metadata | Cursor, multiline, attachments, native key actions and history unaffected |
| **P5 Ghost text** | Render screenshot-style placeholder locally | Disappears on all nonempty inputs; zero text submitted from placeholder |
| **P6 Popup theming** | Polish relocated autocomplete as transparent bordered popover | Same theme and keyboard/mouse behavior on dark/light/custom package themes |
| **P7 Agent activity** | Observe `agent_*`, `message_*`, `tool_execution_*`; present compact progress | Parallel tools correct, errors truthful, no duplicate final assistant text, permission UI undisturbed |
| **P8 Minimal footer** | Directory left; cost + context right; no status/model/effort | Honest metrics, width-safe, updates promptly; no unrequested fields |
| **P9 Integration** | Validate all six installed extensions and project custom themes | All hooks and popup/input flows coexist; no sandbox/permission regressions |
| **P10 Hardening** | Automated unit/integration/property/snapshot tests plus actual Kitty manual checks | No overflow, broken IME/cursor, memory leaks, stale UI, or introduced token use |
| **P11 Delivery** | Ship **only** `opencode-ui.ts` at runtime, remove old adaptive spacer; document settings and rollback | Single file loads and restores gracefully on reload/uninstall |

**Do not describe the finished extension as production-ready merely because it typechecks or renders the mockup.** P1 and the full Kitty matrix are release blockers.

---

## 15. Comprehensive test and acceptance matrix

### Popup-specific (highest risk)

- `/`, `/new`, `/theme`-style built-in + extension commands and commands with arguments; filter/clear text; no matches; extremely long descriptions.
- `@`, `@src/`, nested paths, spaces, Unicode, filenames exceeding terminal width, missing refs; apply completion without unintended submit.
- Up, down, PageUp, PageDown, Tab, Enter, Escape, Ctrl+C, mouse wheel/click, focus and mouse text selection, Kitty keyboard protocol, hardware cursor.
- Menu always directly above editor in HOME and CHAT, with first/last row visible and current selection correctly highlighted. Popover closes predictably and editor never shifts vertically on selection.
- Small screen, terminal resize during open menu, multiline composer, mixed image+text, extra above-editor widgets, model selector/permissions overlay opening while popup active.

### Dashboard / terminal

- Terminal sizes: 60×18, 80×24, 100×30, 120×40, 160×50, and extreme long/short widths. Test Kitty transparency `1.0` and translucent, truecolor and limited-color fallback.
- Fresh start, existing session, CLI initial prompt, `/new`, `/resume`, `/tree` empty/nonempty, `/reload`, multiple concurrent nested agent sessions, large transcript and compaction.
- Theme switches at runtime among `system`, `dark`, `light`, trusted project and installed-package custom themes; logo, prompt, popup, tools/activity, and footer recolor **in the same paint cycle**, with no hardcoded background.
- No startup list of context/skills, but AGENTS context and skill functionality remain intact; warnings/errors still visible.

### Agentic workflow / security

- Successful tool, tool error, blocked tool, aborted tool, nested calls, two+ parallel tool calls, rapid start/end sequences, stale callbacks from replaced session, streaming partial updates, retry/compaction/abort.
- `pi-sandbox` online/offline modes and access request UI; `pi-permission-system` allow/ask/deny; `pi-openui` interactive controls; `pi-cache-hit-monitor` widget expanded; `pi-lens` errors and statuses; `pi-feature-dev` skill invocation.
- Confirm **no change** to permission evaluation, sandbox process constraints, model-facing tools, AGENTS content, MCP, or user data. Any UI event listener must be observational and return `undefined`.

### Footer / billing

- Very long home directory, symlink paths, non-Git directories, Unicode CWD; unauthenticated or nonbilling models; subscription models; missing usage, absent percentage/context window, compacted sessions, nested tool usage, and repeated resumes. No cost duplication.
- LSP and Sandbox status never appear in custom footer. Model/effort appear **only inside** composer (unless another extension independently renders them elsewhere).

### Automated deliverables (kept outside production file)

- Typecheck against the **exact Pi 1.5.2 package API**.
- Unit tests for `home → chat` state changes, geometry, height clamps, directory shortening, cost/usage formatting and ANSI-width text.
- Golden-render/snapshot tests for 5 provided target layouts and both dark/light theme variants, including transparent background assertion (PNG RGBA proof and terminal ANSI `bg` checks).
- Editor/input integration tests exercising real `CustomEditor` and native completion provider, with fake terminal. Mouse y-coordinate mapping assertions. No silent stubbing of the hardest behavior.
- Repeated `/reload` + session switch leak test; 0 duplicate widget keys, no growth in listeners/timers. Trace the active tool set and system prompt before/after to prove no UI-induced change.
- Manual live Kitty sign-off required for release. Record any irreducible upstream limitation in the README; do not call it exact parity.

---

## 16. Delivery, migration, and rollback

**Delivery package for planning:** this Markdown file and `assets/*.png` mockups. **Future runtime installation:** only `opencode-ui.ts` in `~/.pi/agent/extensions/`.

1. Back up `~/.pi/agent/settings.json` and any existing `adaptive-prompt.ts`.
2. Disable/delete the *old adaptive prompt* module before enabling the consolidated UI to prevent two spacers. **Leave all six installed extensions in place.**
3. Merge `tuiMode: "fullscreen"` and `quietStartup: true`. Do not reset your theme or package list.
4. Keep Kitty transparency configuration independent and optional; do not change Kitty configuration automatically.
5. Put the future single `.ts` in the extension directory; `/reload` or restart Pi. Validate the P1 popup gate and security dialogs **before** declaring success.
6. To roll back: disable `opencode-ui.ts`, restore old settings/extension backup, `/reload` or restart. No other package or security setting needs reinstalling.

---

## 17. Open questions and explicit feasibility risks for the implementer

These are not requests for the user to make more decisions now; use the defaults below unless source/API testing forces a change.

- **Autocomplete visual relocation:** mandatory, **highest risk**. Default to preserving native selection and input semantics; request upstream API change if it cannot be made robust with only public APIs. Never substitute a decorative duplicate.
- **Global transparency:** own UI is fully transparent. If an installed third-party theme paints its native tool transcript backgrounds, respect ownership instead of rewriting the theme. Document an optional theme-level adjustment if necessary.
- **True usage cost:** label *estimated* rather than “billed.” Unknown usage must render `—`; no model-only estimator call.
- **Agentic status:** add factual tool activity/progress; no invented progress percent, ETA, or plan step. Leave Pi's core transcript renderer in control of tool output.
- **ASCII Pi style:** original fixed-width logo with semantic theme color, no OpenCode wordmark copy.
- **Animation:** disabled for v1. The dynamic **position transition still happens** without continuous animation.
- **Visibility of footer statuses:** user explicitly removed LSP and sandbox state from the footer. Security-critical overlays/notifications are not suppressed.
- **Runtime configuration:** no extra JSON file needed for this extension; mutable typed defaults near the top of the single `.ts` support future customization.

## 18. Primary references and research links

1. [Pi 1.5.2 extension interfaces, lifecycle, autocomplete provider and tool events](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/extensions.md)
2. [Pi 1.5.2 fullscreen transcript and composer layout](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/chat-viewport.ts)
3. [Pi 1.5.2 interactive startup and `quietStartup`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/interactive-mode.ts)
4. [Pi 1.5.2 `Editor` implementation (autocomplete below, cursor, paste, mouse)](https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/editor.ts)
5. [Pi 1.5.2 `CustomEditor`](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/components/custom-editor.ts)
6. [Pi 1.5.2 `SelectList` (native suggestion list)](https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/select-list.ts)
7. [Pi 1.5.2 theme system, built-in/system/custom/package themes](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/themes.md)
8. [Pi example: custom editor and status](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/border-status-editor.ts)
9. [Pi example: custom footer](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/custom-footer.ts)
10. [OpenCode initial home route](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/home.tsx)
11. [OpenCode composer implementation](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/component/prompt/index.tsx)
12. [OpenCode session and agent workflow](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/session/index.tsx)
13. [Kitty official `background_opacity` and transparency caveats](https://sw.kovidgoyal.net/kitty/conf/#opt-kitty.background_opacity)
14. [Pi sandbox status implementation](https://github.com/carderne/pi-sandbox/blob/main/src/extension.ts)
15. [Pi cache-hit monitor's editor widget](https://github.com/narumiruna/pi-extensions/blob/main/packages/pi-cache-hit-monitor/src/cache-hit-monitor.ts)
16. [Pi OpenUI UI widget integration](https://github.com/newtonapple/pi-openui/blob/main/src/pi/extension.ts)

---

**Implementation authorization:** This document is a **plan only**. No settings were edited and no `opencode-ui.ts` was installed or tested. The next engineering task is a popup-placement feasibility spike on Pi 1.5.2, then the rest of the implementation, followed by full Kitty tests. The mockups define the target but are not proof of delivered behavior.
