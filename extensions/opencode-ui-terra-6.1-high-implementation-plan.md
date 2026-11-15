# OpenCode-inspired Pi UI — production implementation contract for Terra-6.1 High

**Document type:** Executable implementation prompt and acceptance specification. **This is a plan, not the implemented extension.**  
**Research baseline:** 15 November 2026; Pi Coding Agent **1.5.2** (`@earendil-works/pi-coding-agent`), Kitty on Linux.  
**Model to execute this plan:** **Terra-6.1, High reasoning**. Treat this specification as the source of truth; investigate the checked-out Pi APIs before writing code.  
**One runtime deliverable:** `~/.pi/agent/extensions/opencode-ui.ts`. Tests, fixtures, documentation and screenshots may live outside that runtime file; the installed extension must remain a single TypeScript file.  
**Currently installed:** `pi-sandbox`, `pi-cache-hit-monitor`, `pi-openui`, `pi-permission-system`, `pi-lens`, `pi-feature-dev`. `pi-fast-tui` is removed.  
**Illustrations:** The PDF edition contains **18 designed Tokyo Night mockups** at the figure positions referenced below. The Markdown edition intentionally contains **no images**, but otherwise has the same prose, headings, tables and code.

> **Engineering instruction to Terra:** Implement rather than merely explain, but perform feasibility gates before making invasive changes. Preserve user files and existing Pi configuration. Never misrepresent proposed mockups as running screenshots, never fabricate status data, and never declare production readiness before integration tests in Kitty. This plan does **not** authorize changing Pi upstream, installing a new theme package, or changing any sandbox or permission policies.

---

## 1. Mission, priorities and success criteria

Create a coherent, OpenCode-inspired terminal UI for Pi with the following capabilities:

1. **Startup home:** A clean ASCII-art **Pi** symbol near the top-middle, an elegant composer around the upper-middle, and a minimal, bottom-anchored footer. Hide ordinary Pi welcome text, resource inventories and keybinding banners. Keep meaningful startup warnings visible.
2. **Adaptive transition:** On the first *committed* message or shell/agent action, remove the home logo and move the real composer to the bottom. Resume existing nonempty sessions directly in conversation mode. Preserve behavior through `/new`, `/resume`, `/tree`, `/reload` and resizing.
3. **Card-forward presentation:** Normal transcript and activity should look deliberately organized, not like bare ungrouped terminal lines. Tool, diff, test, warning, progress and completed-work cards need distinct **tinted surfaces** and narrow semantic accent rails, with generous spacing and legible typography.
4. **Prompt composer:** Accurate OpenCode-inspired geometry: accent rail, subtle frame, background-tinted input card, true editable multiline input, model and reasoning effort **inside** the prompt, and a purely visual ghost placeholder when empty.
5. **Completion popovers:** Typing `/` or `@` must display an accent-colored, theme-aware suggestion **popover immediately above** the composer, preserving Pi's native completion ranking, file search, keyboard, mouse, IME and submission semantics.
6. **Minimal footer:** **Current directory left; estimated session cost and context usage right.** Nothing else in the footer: no LSP, sandbox, model, effort, Git, spinner or command hints. Genuine sandbox/permission warnings must remain visible in their original locations.
7. **Agentic workflow:** Support richly differentiated display states for planning (only when real), reading/searching, editing/diffs, writing, Bash commands, tests, nested/parallel tools, blocked permissions, errors/retries, reasoning indicators, final answers, summaries and optional verified subagents.
8. **Markdown:** Use native, local Markdown rendering for assistant output; improve its presentation *only through render-only paths*. Headings, inline code, fenced code, tables, task lists, quotes and links must not produce any extra LLM requests or injected context.
9. **Themes:** All production colors must derive from the **currently selected Pi theme**, including custom themes installed from extensions/packages. Tokyo Night is the **illustration palette only**; do not hardcode it as a mandatory runtime theme.
10. **Zero incremental LLM-token use:** No additional prompt/system tokens, model tool schemas, reasoning/model calls, model-visible messages, silent completions, retries or summarization caused by this UI. See §3 for the measurable contract.

**Priority order if goals conflict:** security and faithful input handling → zero incremental model tokens → correct data → API stability → UI responsiveness → visual fidelity. A beautiful fake popup or fabricated workflow is **not** a passing implementation.

### 1.1 Final acceptance statement

The deliverable passes only if one `.ts` file reproduces the home-to-chat transition, native completion appears above, cards remain readable across active themes, Markdown renders locally, all six existing extensions remain functional, and model-visible request payloads are unaffected. A blocked feature must be reported explicitly instead of silently omitted.

---

## 2. Architecture and verified upstream constraints

### 2.1 Pi 1.5.2 source of truth

Inspect **tag `v1.5.2`**, not `main`, before implementation:

- `packages/coding-agent/docs/extensions.md`: `setHeader`, `setEditorComponent`, `setFooter`, `setWidget`, `registerToolRenderer`, `registerMarkdownTransformer`, lifecycle events and custom TUI components.
- `packages/coding-agent/src/modes/interactive/chat-viewport.ts`: fullscreen fixed composer dock beneath a scrolling transcript; above-editor widgets, editor, below-editor widgets and footer occupy separate slots.
- `packages/coding-agent/src/modes/interactive/interactive-mode.ts`: built-in startup header, loaded resources, `quietStartup`, editor replacement, dialog focus and widget limits.
- `packages/coding-agent/src/modes/interactive/components/custom-editor.ts`: supported subclass preserving Pi keyboard actions.
- `packages/tui/src/components/editor.ts`: native text wrapping, cursor markers, paste handling, suggestion placement and mouse coordinates.
- `packages/tui/src/components/select-list.ts`: native autocomplete rows, selection and scrolling.
- `packages/coding-agent/src/modes/interactive/components/tool-execution.ts`: existing tool shell and result composition using Pi theme background roles.
- `packages/coding-agent/src/modes/interactive/theme/theme.ts`, `docs/themes.md` and `docs/tui.md`: semantic theme foreground/background support, Markdown and render lifecycle.
- `packages/coding-agent/examples/extensions/border-status-editor.ts`, `custom-footer.ts`, `custom-header.ts`, `widget-placement.ts` and `message-renderer.ts`: verified patterns to adapt.

**Crucial upstream detail:** The native editor presently appends autocomplete rows **below** its border. Moving them visually is insufficient unless selection, pointer and overlay geometry are also correct. There is no documented `autocompletePlacement: "above"` switch. This is a release-blocking feasibility gate, not a detail to hand-wave.

### 2.2 OpenCode reference behavior

Study the current OpenCode TUI's `routes/home.tsx`, `routes/session/index.tsx`, and `component/prompt/index.tsx`. Borrow interaction and information hierarchy, **not** OpenCode's rendering code or undocumented APIs. The two TUI engines differ. Use native Pi primitives and adapt the visuals.

### 2.3 Screen ownership and boundaries

| Region | Owner in final implementation | Non-negotiable boundary |
|---|---|---|
| Home ASCII mark | `opencode-ui.ts` custom header | No model message; hide in chat |
| Conversation transcript | Pi built-in | Never suppress actual tool results or warnings |
| Workflow cards | Display-only renderer resolver when safe; otherwise read-only widgets | No tool re-registration or command mutation |
| Composer | Pi `CustomEditor` subclass | Preserve editor semantics and focus |
| Completion menu | Pi native completion data, safely relocated | No duplicate command scanner or filesystem crawler |
| Adaptive spacer | One namespaced below-editor widget | No previous `adaptive-prompt.ts` running |
| Footer | One custom `setFooter` component | Do not monopolize other extensions' dialog focus |
| Existing extension widgets | Their existing owners | Respect `pi-openui` and cache monitor |

### 2.4 Required internal organization — one file

Organize the `opencode-ui.ts` source with named sections or internal classes: `SettingsGuard`, `ThemeAdapter`, `TerminalGeometry`, `SafeText`, `SessionPhase`, `Dashboard`, `AdaptiveDock`, `OpenCodeEditor`, `NativeCompletionAdapter`, `CardPrimitives`, `ToolViewResolver`, `WorkflowStore`, `MarkdownPresentation`, `FooterView`, `LifecycleCoordinator` and `Diagnostics`. All are **inside one `.ts`**. Do not introduce runtime imports from custom sibling files or add an npm UI library.

---

## 3. Zero-incremental-token contract (absolute release gate)

**The UI itself must use zero additional LLM tokens.** This is **not** a claim that normal Pi interactions, the selected model, tools, or already-installed extensions consume zero tokens. It means that, with the same Pi session, toolset and user actions, enabling this visual extension does not add model-visible content or trigger extra inference.

### 3.1 Forbidden actions

- Do **not** call `pi.registerTool`, `pi.sendMessage`, `pi.sendUserMessage`, `pi.appendEntry`, `pi.setActiveTools`, provider APIs, MCP APIs, model completion/sampling APIs, or hidden model classifiers for UI work.
- Do **not** add `promptSnippet`, `promptGuidelines`, `AGENTS.md` text, prompt templates, system prompt modifiers, UI-generated chat messages, or automatic summaries.
- Do **not** fetch data by calling a search provider, running another coding agent, or launching a subagent merely to decorate the UI.
- Do **not** turn visual Markdown formatting into an LLM-based rewrite or grammar step.
- Do **not** change tool args, tool results, permission decisions, sandbox policies, model selection or request serialization.
- Do **not** make added cache misses through dynamic tool declaration changes or altered model headers.

### 3.2 Permitted zero-token paths

- Read existing Pi session, model, cost and context metadata using supported APIs.
- Listen to existing events in strictly observational handlers that return `undefined`.
- Use `setHeader`, `setFooter`, `setWidget`, `setEditorComponent` and a **display-only** `registerToolRenderer` only where it is safe to chain via `next()`.
- Use Pi's **built-in Markdown renderer**. `registerMarkdownTransformer` is documented display-only, but it should be **omitted unless needed**; if used, it must be synchronous, deterministic and preserve content semantics.
- Parse already-present result content locally for cards, without changing stored messages or sending it elsewhere.
- Re-render locally on terminal resize, theme change or existing event updates.

### 3.3 Prove the contract, not just assert it

Build a paired, reproducible before/after audit, ideally using a deterministic provider fixture and the same stored transcript:

1. Capture active tool declaration names, schemas, system prompt sections, model messages and request payload signatures with the extension off.
2. Repeat with `opencode-ui.ts` on; assert semantic equality and **zero extra API/model invocations** for identical scripted actions.
3. Verify no extension-owned message entries or model-facing tool schemas are created.
4. Confirm Markdown output, cards, completion and status updates cause zero provider requests.
5. Record **delta input tokens = 0; delta output tokens = 0 attributable to this UI** for the fixed test cases. Allow ordinary non-deterministic model usage variation only outside the controlled fixture.
6. Recheck with `pi-openui`, `pi-feature-dev` and every installed extension enabled; distinguish their existing token costs from the UI's marginal cost.

Any measured nonzero incremental model usage attributable to this extension is a **FAIL**, even if visually acceptable.

---

## 4. Styling system: tinted cards, transparent canvas and inherited themes

The main deficiency in the previous mockups was that panels were visually too flat. This design must make **most meaningful output regions look like cards** with discernible fill and an accent distinct from the surrounding canvas. Do not turn every line into a card: reserve cards for semantically meaningful units.

### 4.1 Tokyo Night visual tokens — PDF illustrations only

| Purpose | Tokyo Night illustrative color |
|---|---|
| Terminal canvas | `#1a1b26` (Night) |
| Card base | `#24283b` (Storm-like raised surface) |
| Card elevated | `#292e42` |
| Selected/hovered | `#343b58` |
| Quiet card border | `#414868` |
| Primary text | `#c0caf5` |
| Secondary text | `#a9b1d6` |
| Muted text | `#565f89` |
| Interactive blue | `#7aa2f7` |
| Accent violet | `#bb9af7` |
| Information cyan | `#7dcfff` |
| Success green | `#9ece6a` |
| Warning amber | `#e0af68` |
| Error red | `#f7768e` |
| Change orange | `#ff9e64` |

Use the source palette from Tokyo Night's maintained terminal themes. The **production extension must not hardcode these hex values**. They are instructions for the PDF artwork and visual acceptance reference only.

### 4.2 ThemeAdapter contract

- Resolve colors **at render time** from the current `theme`/`ctx.ui.theme`, not from a cached launch-time palette.
- Support normal Pi theme foreground roles (`accent`, `text`, `muted`, `dim`, `success`, `error`, `warning`, `border*`, `md*`) and available **background roles** (`toolPendingBg`, `toolSuccessBg`, `toolErrorBg`, `userMessageBg`, `customMessageBg`, `selectedBg`). Pi offers `theme.bg(...)`, `theme.style(...)`, `theme.colors` and theme appearance.
- Compose a small semantic mapping: `surface.default`, `surface.raised`, `surface.selected`, `border.quiet`, `accent.active`, `status.success`, `status.error`, `status.warning`. Use only concrete supported roles; do not invent theme token names and pass them to Pi.
- If a theme maps a background role to terminal default/transparent, retain a prominent border + accent rail, so the design remains legible without forcing opaque colors.
- Theme must update immediately on `/settings`, package-supplied custom theme switching, terminal light/dark changes and `/reload`.
- Preserve high contrast; where theme contrast is poor, reduce decorative effects before reducing legibility.

### 4.3 Card primitives

Implement display-only building blocks, all local and compositional:

| Primitive | Appearance | Used for |
|---|---|---|
| `PanelCard` | Tinted fill, gentle border, margin, optional title | Generic tool output / note |
| `ProgressCard` | Raised fill + blue rail + status glyph | Running tool / grouped actions |
| `SuccessCard` | Raised fill + green rail | Verified passing tests / completed step |
| `WarningCard` | Raised fill + amber rail | Retry or uncertainty; never hide host warning |
| `ErrorCard` | Raised fill + red rail | Tool failure, command exit nonzero |
| `DiffCard` | Split add/remove rows with green/red accents | Existing actual edit/diff result |
| `CodeCard` | Separate muted title bar and monospace body | Read snippets, code/Markdown fences |
| `DecisionCard` | Visually distinct elevated surface | Host-controlled permission UI only, when safe |
| `SummaryCard` | Strong heading and metrics grid | Completion report derived from actual data |

Card borders must survive narrow terminal widths. Use rounded box-drawing characters (`╭╮╰╯`) only where Kitty/font support is verified, with `+|-` fallback. Text must never be hidden by heavy decorative borders.

### 4.4 Transparent background: exact meaning and trade-off

**Canvas transparency and colored card surfaces coexist:** unoccupied terminal cells retain the terminal's default background; the **cards intentionally use themed tinted background cells** for visual contrast. Terminal SGR colors do not provide arbitrary per-cell alpha. Therefore the extension alone cannot promise that *filled card cells* show the desktop through them.

- If the user wants desktop-through transparency, configure Kitty's `background_opacity` separately.
- Kitty also supports `transparent_background_colors` for selected colors on supported versions/compositors. This is **optional**, must be checked against the user's Kitty version, and must not be silently written by the extension.
- The default is an unobtrusive transparent **canvas** with contrast-rich cards. If the user insists on a wholly transparent image, offer an optional **border-only fallback** (less OpenCode-like) rather than claiming per-cell alpha that is not available.
- Do not draw a fullscreen background rectangle in the extension. Do not change terminal background OSC settings, terminal palette or window opacity.

**Figure F01 — Card hierarchy and Tokyo Night surface contrast.** In the PDF, this plate proves the tinted card surfaces are visually distinct from the surrounding canvas.
<!-- FIGURE:F01 -->

---

## 5. Required configuration, optional terminal settings and migrations

### 5.1 Required Pi settings

Merge the following **keys only** into existing `~/.pi/agent/settings.json`:

```json
{
  "tuiMode": "fullscreen",
  "quietStartup": true
}
```

- Preserve existing `packages`, `extensions`, `theme`, MCP, security, provider, model and unrelated settings.
- `tuiMode=fullscreen` is required for the independent transcript and fixed input dock.
- `quietStartup=true` suppresses ordinary startup header/listings, but **does not disable** loaded AGENTS files, skills or extension functionality. Genuine diagnostics may still appear and must not be suppressed.
- **No `theme` change is required.** If the current custom theme is already Tokyo Night, inherit it; otherwise follow the user's chosen Pi theme. The Tokyo Night PDF examples do not authorize changing the runtime theme.
- The extension itself requires no additional configuration keys, external JSON file, npm package, font, or model-facing command. Keep its typed defaults at the top of `opencode-ui.ts` for later manual modification.

### 5.2 Optional Kitty configuration (separate file)

Only for users who actually want the desktop visible through the terminal:

```conf
# ~/.config/kitty/kitty.conf — optional and user-controlled
background_opacity 0.88
# On supported Kitty versions only: selected tinted UI colors can be translucent.
# transparent_background_colors #24283b@0.92 #292e42@0.92 #343b58@0.92
```

Check current Kitty documentation and version before enabling the commented option; color matching and background-opacity support depend on the platform/compositor. The UI must function at `background_opacity 1.0` too. Do **not** enable Kitty remote control or dynamic opacity management merely for this extension.

### 5.3 Migration and rollback

- Back up `~/.pi/agent/settings.json` and any existing local `adaptive-prompt.ts`.
- Disable the old adaptive prompt extension to prevent **double spacer ownership**; do not uninstall the six unrelated extensions.
- Install only the unified `opencode-ui.ts` when implementation and tests are finished; use `/reload` or restart.
- Rollback: remove/disable `opencode-ui.ts`, restore backed-up settings, and `/reload`. No changes to sandbox, permissions, MCP, theme files, or Pi binary should be necessary.

---

## 6. Home dashboard and adaptive positioning

### 6.1 Empty session layout

Use a single ASCII Pi logo (original arrangement, not copied OpenCode trademark artwork). The logo is horizontally centered and placed above the prompt with deliberate whitespace. No Pi version string, CLI usage text, `[Context]`, `[Skills]`, extension inventory or onboarding prose in a normal successful startup.

The composer should appear around **48–55% of the terminal height**, not at the bottom. Footer stays pinned to the bottom. Card styling must be present even on the home screen.

**Figure F02 — Fresh-session dashboard.** ASCII Pi icon, shaded prompt card, in-prompt model/effort, and directory/cost/context footer.
<!-- FIGURE:F02 -->

### 6.2 Session phase state machine

```text
STARTUP -> HOME             when the active branch has no meaningful work
STARTUP -> CHAT             when resuming a nonempty branch
HOME    -> CHAT             on first committed prompt / shell / agent action
CHAT    -> HOME             on /new or /resume(empty) or /tree(empty)
CHAT    -> CHAT             on /resume(nonempty), /tree(nonempty), compaction
HOME/CHAT -> re-evaluate    after /reload or project/session rebinding
ANY     -> SHUTDOWN         on session teardown (dispose owned UI)
```

Do **not** switch to CHAT when the user merely types, selects an autocomplete result, opens `/settings`, changes the theme, or invokes a non-agent UI command. Use `session_start`, `session_shutdown`, `session_tree`, `input`, `before_agent_start`, `agent_start`, and `user_bash` as appropriate, after verifying actual event semantics. Avoid `session_switch`: not part of the inspected 1.5.2 lifecycle.

### 6.3 Layout algorithm

- Measure current TUI `rows`, `columns`, editor height, footer height, popup height and extra widgets on each relevant render.
- Implement one namespaced `belowEditor` spacer component that returns dynamically calculated lines. Do not use string-array widgets to create large spacers: Pi caps array-widget lines.
- Calculate `homeTargetEditorTop`, clamp to free rows, allocate a max spacer, and preserve at least one visible transcript row; clamp margins when screen is short.
- In CHAT, spacer is **zero**. Logo is hidden or empty; editor is naturally docked at bottom. No key-frame animation initially; a discrete layout transition prevents cursor/IME desynchronization.
- Handle narrow and short terminals (e.g. 60×18) with a smaller/no logo and zero home spacer, never hiding the editor or footer.
- Preserve Pi's scroll view, jump-to-latest behavior, focus and subsequent history rendering. Avoid any direct `stdout` ANSI positioning hack.

**Figure F03 — Transition from HOME to CHAT.** A two-state plate showing the same real editor in two placements without a replacement input control.
<!-- FIGURE:F03 -->

---

## 7. Composer design, ghost text and input integrity

### 7.1 Custom editor implementation

Subclass Pi's public `CustomEditor` and **preserve its native input pipeline**. Use `setEditorComponent((tui, theme, keybindings) => new OpenCodeEditor(...))`, respecting prior editor ownership and reinstallation on session lifecycle only when necessary.

Required UI geometry:

- A raised card with a narrow violet/blue rail on the left, a fine theme border, and padding that does not corrupt cursor math.
- Input body with a two-row minimum and correct multiline growth up to Pi's native maximum; don't fake the editor as a `Text` component.
- Metadata strip *inside* the card: interaction mode (`Chat` or actual available mode), model label and thinking effort. These must be true current values, never static mockup strings.
- Preserve native prompt history, autocomplete, bracketed paste, image paste, text selection, undo, multiline entry, IME cursor markers, Kitty keyboard protocol, `/` commands, `@` references and extension shortcuts.
- No new event loop, polling or model-based prediction.

### 7.2 Ghost placeholder (zero tokens)

Display e.g. `Ask anything…  "Fix a TODO in the codebase"` only while the actual editor is empty. Render it as **visual glyphs**, not contents of `getText()`, and do not insert it into the session or provider payload. Hide it as soon as typing, paste, images or an active composition means input is nonempty; restore on clear. Ensure the cursor remains visible and positioned correctly.

This feature is a placeholder, **not** AI-suggested next tokens. No per-keystroke API calls are permitted.

### 7.3 Composer figure and acceptance

**Figure F04 — Composer anatomy.** Shows fill, accent rail, placeholder, model/effort row, focus and multiline height.
<!-- FIGURE:F04 -->

Acceptance tests: 1,000-character wrapping, 100-line paste, Unicode combining characters, emoji/wide glyphs, terminal resize while typing, Ctrl+C/D, Tab/Shift-Tab, prompt history, command autocomplete, images and focused/unfocused state. Any cursor offset, lost paste data or changed submit text fails the implementation.

---

## 8. `/` and `@` suggestion popovers — first implementation gate

### 8.1 Visual requirements

The suggestions appear **above**, directly touching or narrowly separated from the card's top border; never below the composer. The popover is an elevated theme-filled card with border, selected-row accent rail, clear descriptions, and a maximum height proportional to available rows. A bounded popover in HOME must not push the composer off its intended position; in CHAT it must not cover critical dialogs or the footer.

**Figure F05 — `/` command popover.** Native commands and installed-extension commands, selected row and key hints in a visually distinct card.
<!-- FIGURE:F05 -->

**Figure F06 — `@` path popover.** Native file reference suggestions, nested path truncation and selection.
<!-- FIGURE:F06 -->

### 8.2 Behavior to preserve

- `/`: built-in and registered commands, descriptions, ordering, fuzzy matching, empty/no matches, arguments, completion and actual invocation.
- `@`: native file/path lookup, existing ignore behavior, autocomplete insertion and file references. **Do not** create a custom filesystem crawler, read extra source files, or bypass Pi's trust constraints.
- Correct Up/Down, PageUp/PageDown, Tab, Enter, Escape, text editing, backspace, wheel, click, mouse selection, input focus and IME.
- Correct geometry when editor text wraps or becomes multiline, terminal resizes, other widgets are visible, or a dialog opens.
- The popup must use the **same native completion data and insertion actions**; a purely decorative lookalike is not acceptable.

### 8.3 Feasibility gate before broader development

1. Reproduce native editor completion in an isolated fixture against Pi 1.5.2.
2. Determine whether public render and mouse APIs can move the existing `SelectList` rows safely. Because `Editor.render()` appends them below and maintains private height/offset fields, **do not assume simple array row reordering is sufficient**.
3. Prototype a `CustomEditor` subclass that calls `super.render()` and `super.handleInput()` and, if possible, maps pointer y-coordinates to the original list. Test **native state**: selected item, insertion, scrolling, `@`, autocomplete lists with no matches, cursor and focus.
4. If the public subclass interface cannot provide accurate row geometry, test a supported TUI popup/overlay while retaining native completion ownership. Avoid private-field reflection, `as any`, monkeypatches and global stdin capture.
5. If no safe extension-only solution exists, **stop and document a minimal upstream API proposal**, e.g. `autocompletePlacement: "above" | "below"` with an exposed layout/coordinate hook. Do not claim release readiness or ship a broken workaround. Request explicit approval before forking or modifying Pi.

**Gate P1 PASS:** exact native behavior through all tests and popup above. **FAIL:** a decorative popup, broken mouse, focus theft, duplicate scanning or private-Pi patching.

---

## 9. Agentic UI: workflow semantics and visual language

The UI should feel like an OpenCode session: clear message hierarchy, small informative chips, cohesive **colored activity cards**, richly styled output and a stable bottom composer. But Pi still owns planning, model calls, tools, sessions, permissions and transcript. This extension is **an observer and renderer**, not an orchestrator.

### 9.1 Canonical workflow states

```text
IDLE / HOME
  -> PROMPT_COMMITTED
  -> AGENT_STARTING
  -> RESPONDING_OR_THINKING (without exposing hidden reasoning)
  -> TOOL_ACTIVITY [0..N concurrent tools]
      -> WAITING_FOR_APPROVAL (host-owned dialog, if observed)
      -> TOOL_SUCCEEDED / TOOL_FAILED / TOOL_BLOCKED_IF_PROVEN
      -> VERIFYING (only if real test/check tool runs)
  -> RESPONDING_OR_THINKING
  -> COMPLETED | INTERRUPTED | FAILED | RETRYING_IF_PROVEN
  -> IDLE / CHAT
```

Do not invent a “Plan 4/7”, success percentage, tests passed, files changed, price, hidden-chain-of-thought text, ETA, subagent identity or permission decision when no underlying event/result proves it. Distinguish **UI phase** from an actual plan; a model's prose mentioning a plan is not automatically structured planning state.

### 9.2 Event-to-state mapping

| Existing Pi event or stored data | UI action | Truthfulness requirement |
|---|---|---|
| `before_agent_start`, `agent_start` | Show run-start activity and leave HOME | No artificial extra turn |
| `message_start`, `message_update`, `message_end` | Native streaming text and read-only visual status | No duplicated message payload in model context |
| `tool_execution_start` | Create running card keyed by `toolCallId` | Tool has started, not succeeded |
| `tool_execution_update` | Update card with bounded event data | No exhaustive raw logs or per-token redraw |
| `tool_execution_end` | Finalize duration and actual `isError` outcome | Blocked vs error only if conclusively known |
| `parentToolCallId` on nested execution | Show child under parent if provided | No invented agent hierarchy |
| `turn_start`, `turn_end` | Maintain correct turn grouping | Turn ≠ whole task completion |
| `agent_before_settle`, `agent_settled` | Finalize status; clear live-only widgets | Respect retry, compaction and queued follow-ups |
| `session_start`, `session_tree`, `session_shutdown` | Reconstruct, clear or dispose state | No stale cards across sessions |
| Stored assistant/tool-result entries | Build truthful resumed read-only view | No replaying tool actions |

Refer to Pi's latest **v1.5.2** lifecycle docs, including nested calls and delayed settlement. Never rely on event fields not present in the installed version.

### 9.3 Presentation hierarchy

- **Conversation stream:** user prompt card → assistant response (native Markdown) → intervening tool card stack → result/final response.
- **Live activity dock:** at most 3–5 bounded cards or lines above the composer for active steps; when the transcript already shows the same information, show a compact summary instead of duplicating full output.
- **Tool result card:** identity/icon, concise target (sanitized path/command), state, elapsed time (when known), muted details, optional collapsible preview.
- **Structured evidence:** test counts, changed-file list, code diff, filenames and subprocess exit code **only if represented in actual existing results**.
- **No separate UI-triggered commands:** expanding an already captured card may reveal existing data locally, but may not run tests, read files, or call an LLM.

### 9.4 Detailed output design: 18 visual acceptance figures

The following figures in the PDF are **target visual states**. They may contain sample filenames, commands and sample metrics solely to demonstrate arrangement; they are not live claims about the user's repositories.

**Figure F07 — First response and agent activity.** User question, streamed assistant paragraph, active work cards and stable footer.
<!-- FIGURE:F07 -->

**Figure F08 — Search and file reading.** Grouped `find/grep/read` cards, source lines and file paths; colored cards distinguish index/search from actual read output.
<!-- FIGURE:F08 -->

**Figure F09 — Edit and unified diff.** A themed diff card with path heading, hunk context and +/− rows; only actual returned diff text is styled, not fabricated.
<!-- FIGURE:F09 -->

**Figure F10 — Bash command and test execution.** Command card, captured output snippet, pass/fail counts, status and duration. Aggregate test counts appear only when parsable from actual output.
<!-- FIGURE:F10 -->

**Figure F11 — Parallel/nested tool calls.** Multiple concurrent activities with independent state and a parent-child indicator when `parentToolCallId` exists.
<!-- FIGURE:F11 -->

**Figure F12 — Permission/approval interruption.** A high-priority host-owned prompt remains operable and readable above decorative UI; never draw fake approval buttons.
<!-- FIGURE:F12 -->

**Figure F13 — Failure and bounded retry.** Error card, command exit status and factual retry state. Never mark retry success until a new result confirms it.
<!-- FIGURE:F13 -->

**Figure F14 — Markdown-rich assistant response.** Headings, bullets, inline code, fenced code, table, quote and link styled using native Pi Markdown rules and theme roles.
<!-- FIGURE:F14 -->

**Figure F15 — Completed workflow summary.** Actual completed steps, actual changed files and successful tests, with concise evidence and no invented token/cost totals.
<!-- FIGURE:F15 -->

**Figure F16 — Optional subagent/role view.** Render only if **existing structured extension events/results** genuinely identify a subagent, its role and state; otherwise omit the card entirely.
<!-- FIGURE:F16 -->

**Figure F17 — Existing `pi-openui` widget coexistence.** Interactive output is displayed in its original component without being replaced, and surrounding cards maintain hierarchy.
<!-- FIGURE:F17 -->

**Figure F18 — Responsive short terminal.** Small-height and narrow viewport preserve focused composer, scroll history, reduced card density and minimal footer.
<!-- FIGURE:F18 -->

### 9.5 Tool categories and per-type rendering rules

| Tool category | Presentation when already executed | Never do |
|---|---|---|
| `read`, `ls`, `find`, `grep` | Search/read card with path and small preview | Automatically read extra files |
| `edit`, `write` | Change card with truthful path and diff/size if available | Invent patch hunks or duplicate writes |
| `bash` / shell | Command card, exit code/status, bounded stdout/stderr | Execute command again for UI |
| Test invocation | Specialized test card only with evidence | Parse every arbitrary shell line as a test count |
| MCP tools | Generic named tool card with safe metadata | Change MCP bindings or auth |
| Nested/codemode | Parent-child indicator from nested IDs | Treat nested call as independent top-level message |
| `render_widget` (`pi-openui`) | Preserve its own interactive renderer | Capture/redirect clicks or shadow its tool |
| Permission/sandbox | Preserve original dialog/warnings; optional passive waiting label | Bypass/auto-approve requests |
| Unknown/custom tool | Generic fallback, `next()` chain | Assume data schema or tool purpose |

### 9.6 Production renderer integration

- Pi 1.5.2 offers `pi.registerToolRenderer((toolName, next) => ...)`. `next()` chains existing renderers. Investigate whether returned `renderCall`/`renderResult` components can be **safely wrapped** in card shells without changing tool definitions, `renderShell` expectations, focus, or existing custom renderers.
- Preserve third-party renderers in their original order. Do not replace `pi-openui`, security-tool wrappers or unknown interactive components with string summaries. For interactive or `renderShell:"self"` tools, prefer `next()` untouched.
- For tools where safe composition is proven, style with `theme.bg('toolPendingBg'/'toolSuccessBg'/'toolErrorBg')`, borders/rails and text padding. Do not copy tools into new `registerTool` definitions.
- If Pi's renderer chain cannot wrap certain core tool displays safely, use a **read-only adjacent activity widget** and preserve native content. Record visual parity as partial; do not fake complete ownership of the transcript.
- Never decorate raw model/tool content by injecting uncontrolled terminal escape codes; sanitize untrusted identifiers, truncate safely by terminal-visible width, and preserve copied raw output through native Pi actions.

### 9.7 Parallelism, cancellation and session correctness

- Maintain a bounded `Map<toolCallId, ActivityRecord>` and parent IDs; update by ID, not by index/order.
- Parallel starts/ends may interleave; card order should follow first observed start, not pretend serial progression.
- Aborted or permission-blocked calls can still emit `tool_execution_end` with error; inspect evidence before assigning a specialized label.
- Retry after provider failure and auto-compaction can extend one logical workflow across multiple agent passes; do not interpret `agent_end` alone as final completion.
- On `/tree` branch switch, recompute from branch records and remove transient displays from abandoned branches.
- Use timestamps/durations from existing events where available; otherwise display an em dash, never a guessed duration.
- Keep a small bounded history of recent records for active widgets; leave complete durable transcript to Pi.

---

## 10. Markdown rendering with no model usage

**Default:** trust Pi's native Markdown renderer, which already formats headings, emphasis, links, lists and fenced code locally. The extension should not create or re-request Markdown from the assistant. **Do not change the assistant response text or submit it to another model.**

### 10.1 Visual Markdown requirements

- Headings use theme's `md*` color roles; paragraphs have breathing room; code fences appear in visually distinct **code cards** if the relevant public render API safely allows it.
- Preserve syntax highlighting and hyperlinks, including Kitty links/copy behavior where supported.
- Tables remain aligned with correct measured terminal widths; if short, horizontally clip or wrap without corrupting code or active input.
- Respect quotes, bullets, nested lists, checkboxes and inline code. Raw assistant Markdown, not visually enhanced Markdown, stays in stored context.
- Do not syntax-highlight by calling an LLM or fetching any remote renderer assets; Pi's local highlighter is allowed.

### 10.2 Safe APIs and fallback

- Pi 1.5.2 documents `pi.registerMarkdownTransformer` as **display-only**. It can restyle text synchronously, but changes the display content and may be called during streaming, resize and restored history. Prefer not to register it unless there is a demonstrated, tested need.
- Prefer Pi theme's built-in Markdown color roles for normal transcript content and Pi TUI `Markdown` components only for **extension-owned local panels**.
- If an exact background-filled Markdown answer card is impossible through public API without replacing core transcript rendering, preserve native Markdown and improve surrounding layout instead of monkeypatching internal message renderers.
- Markdown previews must be deterministic, bounded, XSS/ANSI-safe, keyboard accessible where possible, and free of network access and added provider requests.

**Zero-token test:** Re-render the exact same Markdown 50 times (theme swap + terminal resize); the provider call count must remain unchanged, and stored session messages must match byte-for-byte.

---

## 11. Footer: exact left/right specification

A single fixed bottom row, with no card filling the entire line:

```text
~/Projects/my-project                       Cost $0.014 est  ·  Context 18% / 272k
```

Left: **current working directory** (prefer `ctx.cwd`, shorten `HOME` prefix at a path boundary, safely truncate/ellipsis for small widths). Right: **reported or estimated session cost**, then **actual context usage**. Right-align the right group; do not invert the order.

### 11.1 Data fidelity

- Context from `ctx.getContextUsage()` and `ctx.model?.contextWindow`, not a hand-maintained counter. Show partial/unknown if unavailable.
- Cost from session assistant usage and documented usage entries without double counting. Label `est` when Pi/model reports estimated API-equivalent dollars; for subscription/unreported values use `Cost —`, never invent billing or claim the user paid this amount.
- Update only from existing session events and a lightweight TUI invalidation, not a polling provider call.
- Compact precedence: preserve context value and directory basename on narrow terminals; optionally shorten the cost if both cannot fit; on tiny viewports use two short lines rather than overlap.

### 11.2 Explicit omissions

**Never show** LSP, Sandbox, model, effort, Git branch, extension list, cache monitor, keybinding hints or working spinner in the footer. Model and effort **already appear inside the composer**. This is a visual omission only: `pi-sandbox`, `pi-permission-system` and `pi-lens` remain installed and their real alerts/dialogs must function.

---

## 12. Compatibility contract for the installed extension stack

| Installed extension | Required coexistence behavior | Validation |
|---|---|---|
| `pi-sandbox` | Keep sandbox enforcement and important prompts; do not change its status state or access requests | Denied write remains denied; approval still appears |
| `pi-cache-hit-monitor` | Preserve above-editor widget without editor/popup collision | Toggle/expand monitor during popup and run |
| `pi-openui` | Preserve `render_widget`, any interactive TUI UI and tool renderer | Buttons/choices/cursor still work; no extra call |
| `pi-permission-system` | Never alter or suppress allow/ask/deny decisions | Test all permission paths and blocked tools |
| `pi-lens` | Keep LSP processes, diagnostics and UI notifications intact, but omit routine LSP footer status | LSP command and failure paths still work |
| `pi-feature-dev` | Keep skill discovery/invocation; `quietStartup` affects only visible resource listing | Skill command remains available |

Third-party themes and future UI extensions may also own `setEditorComponent`, `setFooter` or header slots. Detect conflicts and fail clearly rather than reinstalling repeatedly to win ownership. The removed `pi-fast-tui` must **not** be reintroduced as a dependency.

---

## 13. Operational, security, performance and accessibility rules

1. **No privilege changes:** No changes to sandbox access, shell interception, approval hooks, filesystem scope or network policy.
2. **No extra model usage:** Strict §3 audit. No visual UI may enqueue a message, request a tool, or alter a prompt.
3. **Pure rendering:** Rendering functions must avoid subprocesses, disk scans, network, provider calls, timers and file writes. Caches must be bounded and disposed.
4. **Event throttling:** Coalesce frequent tool-update events; do not redraw more than necessary. Any throttle timer is local, short-lived, cancelable and must not touch model state.
5. **Theme hot reload:** Derived styles must refresh on active theme changes; no hardcoded terminal codes for Tokyo Night in `.ts`.
6. **Terminal dimensions:** Correct on 60×18, 80×24, 100×30, 120×40, 160×50, very long inputs and very long file paths.
7. **Text safety:** Use Pi's Unicode-visible-width and ANSI/truncation helpers; neutralize control characters from untrusted file paths and command names before showing them in decorative labels.
8. **Focus:** Never replace host permission dialogs or mouse/keyboard ownership of `pi-openui` widgets.
9. **Truthful errors:** A visible error remains visible after dismissal of the activity widget. Failed model/provider work may not be marked complete.
10. **No extra dependencies:** No extra npm package, image generator, background worker, browser, local webserver or terminal font installation.
11. **Progressive fallback:** When card shell or popup relocation is unsupported, preserve core Pi reliability; document the specific missing upstream API and stop before claiming complete production readiness.
12. **Shutdown:** Dispose widgets, intervals, event listeners, theme subscriptions, editor factory and footer. No stale handlers after `/reload`.
13. **No visual data leaks:** Don't expose raw environment variables, secrets, or hidden tool metadata in a card just because the event contains it. Use only approved display-safe summaries.
14. **Usability:** Maintain contrast and selection markers in both low-color and truecolor environments; keyboard workflow stays first class.

---

## 14. Engineering execution sequence for Terra-6.1 High

Execute in order. **Each phase must produce tests and a gate result**; do not skip the risky autocomplete step to work on decoration first.

| Phase | Work | Gate to proceed |
|---|---|---|
| **P0 Audit and backup** | Confirm installed Pi tag, Kitty version, all six extension versions, settings, theme selection and existing adaptive extension | Baseline saved, reproduction commands documented |
| **P1 Native popup spike** | Prove `/`/`@` suggestions can move above with real keyboard, mouse and IME semantics using public API | **HARD GATE**: both menus work in HOME/CHAT and during resize |
| **P2 Design primitives** | Build theme adapters, sanitized box/card builder, visible-width geometry, fallback for terminal default backgrounds | Snapshot card contrast, no hardcoded production palette |
| **P3 Home/dashboard** | Clean ASCII logo, quiet startup, responsive centered layout and one spacer | No old resource inventory; warnings preserved |
| **P4 Composer** | Subclass `CustomEditor`, accent rail, raised fill, metadata and ghost text | No input, cursor, selection, paste or history regressions |
| **P5 Above-composer popovers** | Package proven P1 implementation in editor, theme card, keyboard/mouse tests | No duplicate completion searches or focus errors |
| **P6 Read-only workflow store** | Map tool IDs, nested/parallel calls, correct run/turn/settlement states, resuming branch | Truthful states, bounded memory, no stale data |
| **P7 Tool card rendering** | Chain `registerToolRenderer` via `next()` where safe; preserve native interactive tools | Native tool output and security behavior unchanged |
| **P8 Workflow outputs** | Read/search, edit/diff, Bash/tests, retry/error, summary, permission coexistence, optional subagents | All 12 output classes covered or limitations recorded |
| **P9 Markdown** | Keep native Markdown; tune only display-safe theme/markup and extension-owned cards | Zero extra model calls; correct links, tables and fences |
| **P10 Footer** | Directory left, cost and context right, width fallbacks | Accurate data, correct ordering, no forbidden fields |
| **P11 Integration** | Run with six extensions, project custom themes, Kitty truecolor/transparency | No regressions, no hidden permission UI |
| **P12 Token/security audit** | Compare provider payload/tool manifest/messages and calls with UI disabled/enabled | **HARD GATE:** zero incremental tokens/calls |
| **P13 Test matrix and hardening** | Snapshot, integration, property, resize, memory and reload-cycle tests | No leaks, clipping, broken cursor or missing data |
| **P14 Delivery** | Install one `.ts`, document setting merge, rollback and limits | User acceptance screenshots from **real Kitty session** |

### 14.1 Test details and required scenarios

**Editor:** type long ASCII and Unicode, multi-line, bracketed paste, image markers, shell mode, undo, history, Ctrl+C/D, empty submit, `/` commands, `@` paths, focus switches and prompt restoration after dialogs.

**Completions:** built-in commands, extension commands, no matches, selection wrap, long descriptions, fuzzy file paths, emoji filenames, mouse wheel/click, page keys, Tab/Enter/Escape, popup vs 3-line/20-line composer, rapid show/hide and tty resizing.

**Workflow:** one tool, failed tool, blocked tool, parallel tools, nested tool IDs, tool result containing code fences, actual diff, test output, provider error, interrupted run, auto-retry, auto-compaction, final settlement, session fork/tree and late callback from old session.

**Visual:** dark/theme installed by extension, system, light, runtime theme changes, Kitty opacity 1.0/0.88, tinted cards, UTF-8 fallback, 60×18/80×24/120×40/160×50, no clip, correct footer and prompt metadata.

**Security:** no prompt injection from tool card labels, no raw secret dump, real sandbox denial, real `pi-permission-system` ask/deny, live `pi-openui` widget, cache monitor widget, LSP error UI, skill discovery.

**Performance:** no rendering subprocesses or network, event update rate bounded, no extra provider/model requests, no listener/heap growth after 30 `/reload` cycles or 20 session switches.

### 14.2 Build artifact acceptance evidence

Provide in implementation handoff:

- The final **single** `opencode-ui.ts`, with clear internal sections and no companion runtime dependency.
- A compatibility report naming the Pi release and the six tested installed extension versions.
- Test outputs and pass/fail results for each gate, including actual Kitty screenshots, **not just the PDF mockups**.
- A before/after token/provider payload audit with unambiguous zero incremental usage.
- Required/optional settings diff, no automatic modification of security or theme files.
- An honest list of deviations from OpenCode appearance caused by supported Pi APIs and any upstream Pi issue/proposal required for popup/renderer parity.
- Rollback instructions and proof that disabling the extension restores native Pi without losing existing session content.

---

## 15. Definition of done / explicit failure rules

**PASS** only when all statements are true:

- [ ] Home looks like Figure F02; after work starts, editor bottom-docks as F03 demonstrates.
- [ ] Most meaningful outputs have visually differentiated tinted card shells and colored rails, as F01 and F07–F18 demonstrate.
- [ ] Input is Pi's *real* editor; model/effort stay inside composer, ghost text is visual-only.
- [ ] `/` and `@` popups appear above, retain native results/keyboard/mouse/IME, and do not intercept host approval dialogs.
- [ ] Bottom left is current directory; bottom right is **cost then context**; forbidden status labels never appear there.
- [ ] Markdown renders locally and readably; no model-generated restyling, summarization or autocomplete.
- [ ] All styling follows currently active Pi themes; no runtime-enforced Tokyo Night palette, and canvas transparency works when Kitty is configured for it.
- [ ] All six installed extensions work, particularly permission and sandbox enforcement.
- [ ] The only runtime artifact is `opencode-ui.ts`; old adaptive prompt extension is not simultaneously active.
- [ ] Paired token/provider audit proves **zero incremental model calls and tokens** attributable to this extension.
- [ ] Tests and real Kitty screenshots pass for tool cards, permission, error, completion, resize, session navigation and rollback.

**FAIL / STOP AND REPORT** when the public Pi 1.5.2 API cannot safely move autocomplete above; tool rendering requires re-registering security-sensitive tools; model messages/tool schemas change; sandbox or permission behavior weakens; or claims of progress/cost/test success cannot be grounded in source data. Do not silently substitute weaker functionality and mark the task complete.

---

## 16. Implementation authority and restrictions

**You (Terra-6.1 High) are authorized to:** inspect source, edit the planned `opencode-ui.ts` and its test fixtures, propose safe `settings.json` merges, run local tests, produce real terminal screenshots and write honest technical notes. Follow this plan in stages, retaining existing user data and original extension behavior.

**You are not authorized to:** silently fork Pi, alter sandbox/permission policies, uninstall the user's six extensions, replace the selected theme, install an additional UI framework, add an MCP/search service, invoke an LLM for UI, create a second runtime `.ts`, modify AGENTS instructions or inject messages, or claim that a PDF mockup proves implementation.

If an API gate cannot pass, stop that feature, provide a narrowly scoped upstream patch/API request **for approval**, and return the rest of the feasible work without claiming full release readiness. Where the spec permits a visual fallback, label it as a fallback explicitly.

---

## 17. Authoritative references and provenance

These links are research inputs, not additional runtime dependencies:

1. Pi 1.5.2 extension interfaces: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/extensions.md
2. Pi 1.5.2 fullscreen viewport: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/chat-viewport.ts
3. Pi startup, widgets and editor binding: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/interactive-mode.ts
4. Pi native editor: https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/editor.ts
5. Pi suggestion list: https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/select-list.ts
6. Pi theme roles and supported formats: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/themes.md
7. Pi tool execution renderer: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/components/tool-execution.ts
8. Pi custom editor example: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/border-status-editor.ts
9. Pi custom footer example: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/custom-footer.ts
10. OpenCode home route: https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/home.tsx
11. OpenCode session route: https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/session/index.tsx
12. OpenCode prompt component: https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/component/prompt/index.tsx
13. Tokyo Night maintained palette: https://github.com/folke/tokyonight.nvim
14. Tokyo Night reference palette: https://tokyonight.org/palette/
15. Kitty opacity/background documentation: https://sw.kovidgoyal.net/kitty/conf/#opt-kitty.background_opacity
16. Kitty per-color transparency: https://sw.kovidgoyal.net/kitty/conf/#opt-kitty.transparent_background_colors
17. `pi-sandbox` event/status integration: https://github.com/carderne/pi-sandbox/blob/main/src/extension.ts
18. `pi-openui` widget implementation: https://github.com/newtonapple/pi-openui/blob/main/src/pi/extension.ts
19. `pi-cache-hit-monitor` widget: https://github.com/narumiruna/pi-extensions/blob/main/packages/pi-cache-hit-monitor/src/cache-hit-monitor.ts

**Research status:** Pi API details above were checked against version-anchored repository code and docs; OpenCode and Kitty/Tokyo Night links describe design or terminal behavior. Figures are illustrative, **not measured behavior**. The next task is implementing and testing the extension, not modifying this plan's conclusions to fit an unverified workaround.
