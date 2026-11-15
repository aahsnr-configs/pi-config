# Current implementation correction

The previous below-popup feasibility inference is withdrawn. The rebuild uses
public protected border hooks to identify native rows and public handleMouse
retargeting to preserve native selection geometry. Complete native call/result
components now use documented renderShell:self. See [REBUILD.md](REBUILD.md) and
the 24 passing checks for current evidence. The earlier research record follows.

# Public API research — 15 November 2026

Latest visual revision: the user requested Pi's original icon and two-thirds
startup position. Installed `pi-logo.js` is not publicly exported, so its 4×2 brand
pixels are expressed using public `rgbColor`/`styleText` primitives. Brand colours
are the explicit logo exception; all decoration and card colours remain themed.
`Theme.style` with `theme.colors[role]` provides concrete filled surfaces without
transparency handling. See VISUAL-REVISION.md for updated screenshots and scope.

Research used the connected Exa app and Context7 during development only. Neither
is a dependency of the extension. No private source, transcripts, credentials or
private paths were submitted to research tools.

Installed package: `@earendil-works/pi-coding-agent` **1.5.2**. Launcher:
`/var/home/ahsan/.pi/agent/bin/pi`, selected by the managed installation's
`current-version` file. Official tag `v1.5.2` resolves through the GitHub REST API
to **b4afdddf44e059c67ccf5097e5317d913521b470**. No Git commands were used.
Four tagged source files were downloaded read-only into `/tmp/opencode-ui-tag`;
all implementation signatures were also checked against installed `.d.ts`/`.js`.

## Exa discovery record

Each row was retrieved on **2026-11-15**. Searches used five results each.
An initial search call lacked the connector's required `objective` parameter;
that call failed validation and is not counted as research evidence.

| Query / concern | Best primary source | Finding and second official check |
|---|---|---|
| `Pi v1.5.2 public CustomEditor autocomplete placement registerToolRenderer Box native Markdown issue 5834` — completion | [Tagged editor](https://github.com/earendil-works/pi/blob/v1.5.2/packages/tui/src/components/editor.ts) | Native render appends suggestions after the bottom border. Mouse routing uses private visible-line/list-height fields. Installed `pi-tui/dist/components/editor.d.ts` makes these private; [tagged extension types](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/core/extensions/types.ts) expose no placement switch or native list getter. Choose SAFE NATIVE BELOW. |
| Same query — cards and renderer chaining | [Tagged message-renderer example](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/examples/extensions/message-renderer.ts) | `Box` with theme backgrounds is supported. [Tagged tool-execution component](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/components/tool-execution.ts) and installed types confirm separate call/result slots and shared state/lastComponent. Composition must preserve native slots and third-party interaction. |
| Same query — stock assistant scope | [Issue 5834](https://github.com/earendil-works/pi/issues/5834) | Exa reported closed/no-action, updated 2026-06-19. The issue is background, not API authority. [Tagged assistant component](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/components/assistant-message.ts) directly uses native Markdown; tagged types scope message renderers to custom messages. Preserve native assistant output. |
| `OpenCode TUI home prompt session component source Kitty background_opacity transparent_background_colors Tokyo Night palette` — design reference | [OpenCode home](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/home.tsx), [session](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/routes/session/index.tsx), [prompt](https://github.com/anomalyco/opencode/blob/dev/packages/tui/src/component/prompt/index.tsx) | Read as hierarchy/interaction references only. OpenCode uses its own Textarea/Box engine; Pi's [tagged fullscreen viewport](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/modes/interactive/chat-viewport.ts) supplies the compatible fixed dock. No OpenCode implementation copied. |
| Same query — transparency/palette | [Kitty configuration](https://sw.kovidgoyal.net/kitty/conf/#opt-kitty.transparent_background_colors), [Tokyo Night](https://github.com/folke/tokyonight.nvim) | Kitty controls transparency externally. Theme-filled cards and the default terminal canvas are different surfaces. [Pi themes](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/themes.md) and installed Theme types supply semantic foreground/background roles. No palette, opacity or terminal configuration changes. |

Direct Exa fetches read the tagged editor, types, message-renderer example, issue,
OpenCode's three component sources, Kitty documentation and Tokyo Night source.
Installed local Markdown/theme code confirmed deterministic local rendering;
there is no Markdown transformer or model-based presentation step in the UI.

## Context7 record

Resolved official library **Pi** to `/earendil-works/pi`. The resolver advertised
versions through 1.4.0, not a guaranteed 1.5.2 snapshot. These three focused queries
returned current/main documentation and were used only for discovery:

1. `CustomEditor setEditorComponent render autocomplete placement above native completion mouse coordinates public API`
2. `registerToolRenderer next renderShell renderCall renderResult preserve component chaining Box themes lifecycle invalidate`
3. `Theme bg toolPendingBg CustomEditor renderTopBorder renderBottomBorder theme hot reload invalidate Component Box Markdown`

Confirmed against installed 1.5.2: `getEditorComponent`, `setEditorComponent`,
`setHeader`, `setFooter`, factory widgets with `belowEditor`, `registerToolRenderer`
and its `next()` chain, `ToolRenderContext.lastComponent`, `Theme.fg/bg`,
`CustomEditor` protected border hooks, public padding, native pointer handlers,
`agent_settled`, nested `parentToolCallId`, and blocking UI prompt events.

## Decisions established by installed code

- `input` handlers run sequentially before later handlers can consume input.
  Commit HOME only on confirmed agent/shell activity. The SDK test includes a
  downstream handler consuming a local UI action.
- Native editor border hooks preserve row/column and autocomplete pointer math.
  Metadata shares the bottom border with the actual hidden-line count. No extra
  input rows or relocated suggestions are introduced.
- `createReadTool()` returns an execution wrapper without renderer callbacks.
  InteractiveMode supplies `withBuiltInRenderers()` before resolving the chain.
  Tests exercise that exact installed renderer path, not a synthetic substitute.
- Existing static `Text` call components can be boxed while preserving their
  content, subclass caches, `state`, and unwrapped `lastComponent`. All result
  callbacks remain identical; self-rendered/interactive/unknown components pass
  through. Missing renderers retain native fallbacks.
- Pi's private fake-cursor boundary markers must survive decoration. Appending
  ghost text by replacing only trailing blanks retains both boundary markers;
  slicing at a visible-column boundary did not. The corrected behavior has a
  regression assertion and was viewed in a real Kitty window.
- Native `@` completion requires an `fd` backend. The integration test supplies
  installed `/usr/bin/fd` and asserts actual emoji matches and no-match behavior.
- Public APIs do not expose all other widgets' measured heights or getters for
  pre-existing custom header/footer factories. Layout reserves room conservatively;
  future prior header/footer ownership cannot be fully preflighted. See REPORT.md.
