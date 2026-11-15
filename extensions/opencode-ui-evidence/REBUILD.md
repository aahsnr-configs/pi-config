# Rebuilt UI: native completion above, complete tool cards

This report supersedes the earlier below-popup/call-heading implementation and
its visual assessments. Implementation follows
`opencode-ui-terra-6.1-high-implementation-plan.md`, with the user's explicit
**no upstream Pi patches** boundary. The previously requested stock Pi logo,
two-thirds startup composer and solid surfaces are retained rather than the
older plan's ASCII mark/middle-height/transparency targets.

Runtime: [opencode-ui.ts](../opencode-ui.ts), one file, installed Pi **1.5.2**.
The prior runtime and tests are saved under `pre-rebuild/`.

## The two architectural changes

**Native autocomplete above the composer.** `CustomEditor.render()` still creates
the real input and real suggestion rows. The protected bottom-border hook marks
their boundary. The custom editor reorders those existing rows and maintains an
explicit visual-row → native-row map. `handleMouse()` delegates with the mapped
row; keyboard actions and completion insertion remain inherited. Pi's own cursor
marker moves with the input, so the TUI positions its hardware/IME cursor at the
actual displayed row. No private fields, duplicate provider, filesystem crawler,
global stdin capture, overlay or artificial suggestions are used.

Native list height is capped at **three visible choices**, preserving native
scrolling/paging and its remaining-item counter. This cap is applied through the
public setter before list creation, so an open menu remains bounded during
resize. Below 24 rows, the menu omits decorative frame rows. The composer includes
a second minimum display row; clicks there map to the real input. It stays real
editable multiline input, not a substituted text component.

**Complete tool cards.** The old approach put a frame around only a call heading
inside Pi's coloured shell. The rebuilt resolver uses documented
`renderShell: "self"` and one card containing both native call and native result
components. Results appear once inside that card, with original arguments,
code highlighting, diff, expansion, errors, image handling and stored result
content retained. Weak row-local caches preserve native `state` and native
`lastComponent` semantics. Native result callbacks still execute; the separate
result slot returns zero display rows because its component is already in the card.

Read, search/list/find, write, shell and edit receive complete frames, semantic
rails, status and actual duration when supplied. No test counts, elapsed times,
subagent identities or success outcomes are invented. Existing interactive
children retain actual mouse dispatch targets through public Container/Box
layout. Unknown tools, `render_widget`, and unrelated self-rendered tools are
left to their existing renderers.

## Evidence

`node opencode-ui-evidence/check.mjs` — **PASS: strict TypeScript; 24/24 tests**.
See [checks.log](checks.log).

The checks include original input/Unicode/paste/submission/history-action parity,
native slash and fd-backed emoji completion, above-row identity, native selection,
paging, wheel/click insertion, resize, cursor marker positions, menu dismissal,
interactive renderer event targets, result visibility/uniqueness, native renderer
caches, lifecycle disposal, footer cost/context, theme changes and the actual Pi
TypeScript extension loader.

The new **actual `createChatViewport` / `renderLayoutFrame` test** opens a native
menu on a long multiline prompt, configures a three-line monitor plus its gap,
native working status and footer, then resizes through 30→18→40→18 rows at width
60. It asserts that the cursor, model/effort, monitor and footer all remain visible.

`python3 opencode-ui-evidence/capture-kitty.py` — real installed Pi InteractiveMode
in an isolated Kitty session, using a deterministic local provider and `/tmp`
fixtures. It captures only its identified window and closes it afterwards:

- [Home](kitty-home.png)
- [Real slash completion ABOVE](kitty-popup.png)
- [Real file completion ABOVE](kitty-file-popup.png)
- [Complete read/edit/error cards and native Markdown](kitty-chat.png)
- [Native confirmation dialog](kitty-dialog.png)

These are running terminal captures, not mockups. The dialog is a fixture
confirmation, not a fabricated sandbox approval. Screenshot geometry is recorded
in `kitty-captures.json`. No user terminal configuration or theme was changed.

[token-audit.json](token-audit.json): actual Pi SDK paired runs remain **2 provider
calls disabled / 2 enabled**, with equal semantic request, tool manifest and
message signatures. **Extra input tokens = 0; extra output tokens = 0; extra
calls = 0** in the controlled fixture. Fifty redraws preserve stored messages
byte-for-byte. No model-facing tools/messages/prompts or runtime network calls
were added.

[no-patch-audit.json](no-patch-audit.json): **zero core Pi file changes** against
the original installed-package hash baseline. Existing concurrent settings/npm
lockfile changes are preserved, as previously documented. No Git/gh operations,
provider changes, sandbox changes or permission changes were performed.

## Gates and honest limits

P1 is now implemented above and passes automated native-state/mouse/layout tests;
the old claim that public APIs categorically prevent relocation is withdrawn.
The missing placement switch did not rule out protected render hooks plus exact
pointer remapping. Manual physical IME candidate-window testing remains open.

P2–P7 and P9–P10 have focused automated/Kitty evidence for their implemented
surfaces. Stock assistant Markdown/container remains Pi-owned, as §10.2 permits;
the UI does not create duplicate assistant messages to simulate answer cards.
No replacement permission controls or synthetic workflow-summary metrics are added.
The active theme controls card colours, so exact Tokyo Night colours are not
claimed when another theme is selected.

**NOT TESTED / release gates still open:** end-to-end all-installed-extension
sandbox/allow/ask/deny/LSP/openui/cache interactions; live billed-provider audit;
physical IME/image paste/Kitty keyboard protocol matrix; actual 30-reload heap and
20-session stress profiles. The original installed compatibility versions and
concurrent environment change are documented in `pre-rebuild/REPORT.md`.

Accordingly this is a rebuilt implementation with verified above menus and full
tool cards, **not certification that every production acceptance gate has passed**.
The user running-session reload was not automated. Use `/reload` from a normal
project; do not start Pi in the agent configuration repository.

Rollback: move `opencode-ui.ts` out of autoload and restore the previous runtime
from `pre-rebuild/opencode-ui.ts`; or disable it and restore the old adaptive
spacer from `adaptive-prompt.ts.disabled`. No settings rollback is needed. Do not
overwrite concurrent package additions with the older settings backup.
