# Screenshot-reference revision — 15 November 2026

The latest user instructions supersede the original middle-height/ASCII-logo and
transparency targets. Card arrangements and proportions in the 16 reference images
are design guidance, not exact layout requirements.

Implemented in `opencode-ui.ts`:

- The dashboard reproduces **Pi's stock 4-column, 2-row brand mark**, with the same
  glyphs and coral/blue/yellow brand colours. Its decorative rounded frame and star
  use the active theme. Pi's logo helper is private, so the extension expresses the
  same pixels through public colour primitives; it does not import private modules.
  Tests compare the mark with the installed stock helper and its colour sequences.
- The startup composer's top row targets `floor(terminalRows × 2/3)`. The calculation
  subtracts actual rendered editor/completion height and footer height; long input
  or a large popup takes precedence when there is insufficient space. Other
  extensions' unknown below-editor widgets can still affect the final position.
- The composer has a rounded frame, a thick accent rail and a neutral solid fill.
  Its native input/cursor/pointer geometry and visual-only placeholder are preserved.
- Extension-owned cards now have rounded borders, padding, neutral solid fill and
  semantic accent rails. Read uses the theme's Markdown-link accent; other work
  uses the theme accent; failed calls use error; completed shell calls use success.
- Background roles are resolved to concrete theme colours, even when a role means
  terminal default. There is no transparency management, OSC background command,
  global palette replacement or change to the user's terminal configuration.
- The current theme remains authoritative. Only the stock icon's brand colours are
  fixed. Exact Tokyo Night colours require selecting a corresponding Pi theme;
  this task did not switch or install one.

## Feasibility against the supplied references

| Reference | Extension-only result |
|---|---|
| 1: card system | Rounded neutral cards and coloured rails implemented on owned/static tool-call surfaces. Native surrounding tool shells retain Pi's status backgrounds. |
| 2: dashboard | Stock Pi icon with decoration; composer starts near two-thirds height. The illustrated ASCII wordmark is intentionally disregarded. |
| 3: adaptive dock | Same real editor; home spacer disappears after confirmed work. Native assistant/user containers keep their host presentation. |
| 4: composer | Filled frame, accent rail, real model/effort and zero-token ghost text. Existing native editable-row geometry remains intact. |
| 5–6: completion | Real native results and interactions preserved below the editor. An above-editor card is not implemented through private geometry or a duplicate search. |
| 7–8: reading/searching | Native tool content, paths and read ranges retained inside decorated call cards; activity remains observational. |
| 9: diff | Actual native edit/diff content preserved. Its self-rendered shell remains native. |
| 10: shell/test results | Shell call card decorated, actual results retained. No fabricated test counts, summary grids or new test execution. |
| 11: parallel/nested | Bounded state keyed by actual tool IDs and parent IDs. Screenshot-specific column arrangements are not reproduced. |
| 12: permissions | Real host-owned dialogs keep focus and appearance; no decorative approval controls. |
| 13: failure/retry | Error accents and original error output persist; the UI does not create retries or infer success. |
| 14: assistant Markdown | Native local Markdown preserved. A replacement card around every assistant response remains unavailable through the selected public hooks. |
| 15: completion summary | No synthetic completion dashboard: metrics require actual structured evidence. Native answers/results remain authoritative. |
| 16: subagent tree | No invented identities or roles. Existing nested tool IDs are supported; a screenshot-style role tree is not fabricated. |

## Verification

`node opencode-ui-evidence/check.mjs`: **strict TypeScript PASS; 20/20 tests PASS**.
The new tests check the original Pi mark and exact two-thirds positioning at
18/24/30/40/50/90/180 rows, including native popup growth and short-screen fallback.
Existing input, completion, mouse, renderer, lifecycle and token checks still pass.

`python3 opencode-ui-evidence/capture-kitty.py`: **PASS**. Refreshed real Pi SDK /
Kitty captures: [home](kitty-home.png), [completion](kitty-popup.png),
[tools/diff/error/Markdown](kitty-chat.png), [dialog](kitty-dialog.png).
Only the isolated fixture window uses `background_opacity=1.0`; no user config was
changed. Its compositor-selected dimensions are recorded in `kitty-captures.json`.

The controlled provider audit still reports **zero extra calls/input/output tokens**.
The package audit still reports **zero changed core Pi files**. Previous limitations
for full-stack security/provider integration, physical IME/image paste and live
reload profiling remain **NOT TESTED**. The broader release classification in
REPORT.md therefore remains NOT READY; these visual changes do not certify it.
