# OpenCode-inspired Pi UI implementation handoff

**Delivery classification: NOT READY for the full release acceptance contract.**
The extension is implemented and its focused checks pass. The complete live
installed-stack security/provider audit and the full interactive acceptance matrix
remain unverified. This is not a claim of FULL EXTENSION-ONLY or pixel parity.

Runtime deliverable: [opencode-ui.ts](../opencode-ui.ts), a single TypeScript file.
Latest appearance changes and the assessment of all 16 supplied references are in
[VISUAL-REVISION.md](VISUAL-REVISION.md). They replace the original ASCII-logo,
middle-height and transparency targets: the dashboard now uses Pi's stock brand
mark with decoration, and the composer targets two-thirds terminal height.
The old spacer was backed up and reversibly renamed to
`opencode-ui-evidence/adaptive-prompt.ts.disabled`, preventing duplicate spacers.
Other existing extension files were preserved. No theme/security/MCP settings
were edited. `tuiMode: fullscreen` and `quietStartup: true` were already configured.

## Implemented behavior

- Empty branches show Pi's original coloured pixel mark in a decorative frame and a raised real composer; confirmed
  agent/shell activity hides the mark and removes the home spacer. UI-only input
  consumed by another extension keeps HOME intact. Nonempty branches resume in CHAT.
- `CustomEditor` retains Pi's input/keyboard/mouse pipeline. One native padding cell
  supplies the rail. Ghost text replaces display blanks only. Model and current
  effort occupy the lower frame, including when multiline input is scrolled.
- Existing static `Text` call renderers for read/grep/find/ls/write/bash receive
  actual themed Box cards. Arguments, expanded source, native component caches and
  results are preserved. Interactive, unknown, missing and self-rendered renderers
  keep their native behavior. Edit/diff presentation remains native.
- A bounded observational activity widget summarizes actual active calls and
  nested IDs, disappears during host dialogs, and does not fabricate permissions,
  plans, progress percentages, tests, subagents or final success.
- Footer contains only cwd and cost followed by context. Unknown/OAuth/Kimi
  subscription cost is an em dash; reported cost is explicitly estimated. Native
  usage records for assistant/tool/compaction/branch-summary/other usage are counted
  once each. Narrow terminals retain context and directory before cost.
- UI colors use active Pi roles; the stock logo preserves its original brand colours. Native assistant Markdown remains untouched;
  no custom messages, transformer, tools, prompts or model calls are registered.

## Commands and actual outcomes

`node opencode-ui-evidence/check.mjs` — **PASS**, strict TypeScript and **20/20 tests**.
Full output: [checks.log](checks.log). The harness uses existing TypeScript 5.9.3;
override `PI_TEST_TYPESCRIPT` and `PI_TEST_MODULES` if those installation paths move.
No compiler, dependency or UI framework was installed, and no live package build ran.

The tests cover real native editor parity, 1,000-character wrapping, 100-line
content/paste, Unicode, cursor boundaries, submission, native slash completion,
real fd-backed emoji file completion/no matches, paging/mouse, metadata while
scrolling, whitespace shell mode, session phases, ownership replacement, 30
cleanup/reinstallation cycles, bounded workflow state, native renderer chaining,
the actual TypeScript extension loader, cost/geometry and dark/light theme changes.

`python3 opencode-ui-evidence/capture-kitty.py` — **PASS**, four real screenshots
of installed **Pi 1.5.2 InteractiveMode in Kitty 0.50.0**. The deterministic local
SDK fixture used `/tmp` files and no user credentials, installed global extension
stack or external model. It closed its own window after capture. The compositor
chose its actual window dimensions; those are in [kitty-captures.json](kitty-captures.json).

- [Home](kitty-home.png): composer near two-thirds height, stock logo, minimal footer.
- [Completion](kitty-popup.png): Pi's real native slash list below the composer.
- [Tool/diff/error/Markdown](kitty-chat.png): successful read, actual edit diff,
  missing-file failure and native Markdown table/code/link.
- [Native dialog](kitty-dialog.png): host confirmation owns focus and replaces the
  editor temporarily. This is a fixture dialog, **not evidence of sandbox approval**.

Screenshots are unedited terminal captures. An external compositor notification
may appear; it is not part of the extension. Text snapshots at 60×18, 80×24,
100×30, 120×40 and 160×50 are in [snapshots.txt](snapshots.txt).

## Token and no-patch evidence

[token-audit.json](token-audit.json) records a paired run using the actual Pi SDK
and its native deterministic faux provider: **2 provider calls off, 2 on**, identical
semantic request signatures, tool names/descriptions/schemas, model messages and
usage. **Fixture delta input tokens = 0, output tokens = 0, calls = 0.** Fifty local
redraws cause no requests and leave stored messages byte-for-byte unchanged.
Dynamic API guards reject unapproved registrations/actions; source checks also
reject model-facing mutations, network/process/filesystem APIs and private access.
Development research/review tokens are outside this runtime measurement.

[no-patch-audit.json](no-patch-audit.json) and the before/after manifests record
**12,018 pre-existing files**. **All 12,017 files other than the npm lockfile match**,
including every enumerated managed Pi release file and the existing extension
sources. Pi source, binary, packages, prototypes and private fields were never
modified or monkeypatched by this task. No Git/gh commands were executed.

**Concurrent environment change:** while this task was running, the settings
packages list and `npm/node_modules/.package-lock.json` changed to add
`npm:pi-optimize-prompt`. This task ran no package install and made no settings
edits. The concurrent change is preserved. A completely unchanged-environment
baseline cannot be claimed; the new package is outside the compatibility baseline.

## Installed compatibility inventory

These versions were inspected, **not all exercised end-to-end together**:

| Package | Version | Evidence / limitation |
|---|---|---|
| pi-sandbox | 0.8.2 | Source unchanged; enforcement and live deny path NOT TESTED |
| @narumitw/pi-cache-hit-monitor | 0.1.4 | Source unchanged; independent widget ID; live expand/collision NOT TESTED |
| pi-openui | 0.3.1 | Its tool name and interactive renderers pass through; live buttons/focus NOT TESTED |
| @gotgenes/pi-permission-system | 42.0.0 | Source unchanged; no permission hooks/results modified; live allow/ask/deny NOT TESTED |
| pi-lens | 4.5.0 | Source unchanged; no LSP/footer status state modified; live diagnostics NOT TESTED |
| pi-feature-dev | 1.29.0 | Installed package supplies skills, no runtime extension entrypoint; discovery unchanged; live invocation NOT TESTED |
| @d3ara1n/pi-command-palette | 0.10.1 | Preserved; no direct header/footer/editor setter found |
| pi-extmgr | 0.4.2 | Preserved; its automatic workflows include Git-capable paths; not started by this task |
| @sreetej510/pi-prompt-manager | 0.14.4 | Preserved; no direct header/footer/editor setter found |
| pi-optimize-prompt | Added concurrently | Outside baseline; version/API/token interaction NOT TESTED |

The inspected original package sources contain no direct calls replacing the
editor/header/footer. This is a static compatibility check, not runtime proof.
Full-stack automation was not run: it could invoke existing extensions' network,
Git or persistent-state workflows outside this task's constraints. No such
workflows or security rules were changed to make a test pass.

## Phase gates

| Phase | Result |
|---|---|
| P0 research/baseline | PASS research and backups; concurrent settings/npm-lock addition recorded |
| P1 native popup | SAFE FALLBACK — native below; public geometry cannot safely relocate it |
| P2 theme/card primitives | PASS installed-native fixtures and dark/light checks |
| P3 dashboard | PASS fixture phase/geometry checks and real Kitty capture; full branch navigation matrix partial |
| P4 composer | PASS automated parity checks; physical IME/image paste/Kitty keyboard matrix NOT TESTED |
| P5 completion | PASS native slash and fd-backed @ checks; above placement intentionally unsupported |
| P6 workflow store | PASS bounded/interleaved/nested/interrupted/cleanup checks; no fabricated workflow |
| P7 tool cards | PASS actual installed native Text renderers, loader and real read/error capture; native interactive/self fallbacks |
| P8 workflow outputs | PARTIAL — read/write/search/shell cards and native diff/error paths; live retry/test/permission/subagent matrix NOT TESTED |
| P9 Markdown | PASS native actual SDK answer and local re-render audit; full Markdown corpus NOT TESTED |
| P10 footer | PASS documented usage aggregation, unknown/subscription policy and width checks |
| P11 full stack | NOT TESTED end-to-end |
| P12 token/security | PASS controlled request equality; live installed-stack provider/security gate remains OPEN |
| P13 hardening | PASS focused checks; exhaustive memory/physical-input/resize/theme-opacity matrix remains OPEN |
| P14 delivery | Single runtime file and rollback supplied; full release acceptance remains NOT READY |

## Explicit visual and API deviations

1. **SAFE NATIVE BELOW** autocomplete. Native list geometry is private; neither
   reordering rows nor an overlay would preserve its public input ownership safely.
2. **Native assistant Markdown/container**, with no fake or duplicate messages.
3. **One native editable row minimum**, rather than the requested two-row minimum;
   added rows would change native list/pointer geometry. Multiline input still grows.
4. **Two-thirds home spacer**: public APIs do not expose other below-editor widget
   heights. Pi can shrink decorative widgets on short screens. Exact placement
   with every possible widget/dialog arrangement is not proven.
5. **Static Text call cards only**; result renderers retain their native shell.
   Self-rendered edit/diffs and interactive/unknown content remain host-owned.
6. **Ownership**: existing editor factory is detectable and causes a clear refusal.
   Later header/footer replacement is detected through disposal and never fought.
   A prior third-party header/footer factory cannot be inspected through 1.5.2's
   public API. The inspected baseline has no direct setter conflicts.
7. **Tiny widths below five cells**: installed Pi itself can overflow a one-cell
   end-of-line cursor and recurse for a wide grapheme. Tests verify that baseline
   explicitly and retain strict width checks for usable viewports. Pi was not patched.
8. Subscription detection is conservative for public OAuth/Kimi metadata. Other
   providers' reported costs remain API-equivalent estimates, not billing proof.

**NOT TESTED:** real sandbox allow/ask/deny, full pi-openui/cache/LSP coexistence,
full-stack request/billing equality, physical IME/image paste and keyboard protocol,
30 actual `/reload` heap profiles, 20 actual session-switch stress runs, custom
extension themes, Kitty opacity 1.0 versus 0.88, and the complete resize/mouse matrix.

## Loading and rollback

The runtime file is in Pi's normal extension directory. This task did not reload
the user's running Pi session. Load it with `/reload` from a normal project when
ready to test; never start Pi in this agent configuration repository.

Rollback requires no session or package changes: move `opencode-ui.ts` outside the
autoload directory and restore `opencode-ui-evidence/adaptive-prompt.ts.disabled`
to `adaptive-prompt.ts`, then `/reload` from a normal project. The existing spacer
backup also remains in `backup/adaptive-prompt.ts`. **Do not restore the settings
backup wholesale over concurrent package additions**; no settings rollback is
needed for this task. No session data was written by the runtime UI.

The isolated SDK disposal tests restore all owned slots and preserve another
extension's replacement slots. Disabling the UI registers no model-facing entries
to clean up. A live full-stack rollback smoke test remains NOT TESTED.
