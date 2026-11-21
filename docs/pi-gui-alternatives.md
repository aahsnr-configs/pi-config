# Pi Coding Agent Frontends, Phosphor Extensions, and KittyBrowser

**Comprehensive chat-session summary — 20 November 2026**  
**Scope:** Phosphor's integration with Pi and `~/.pi/agent`; its automatically loaded extensions; candidates for a less intrusive Phosphor alternative; and the use of KittyBrowser to render rich interactive content inside Kitty.

> **Evidence standard:** This document distinguishes documented/source-verified functionality from reasonable technical possibilities and unresolved compatibility questions. A feature described in a project's README was **not** necessarily exercised locally. The conclusions reflect the research and code inspection performed in this conversation; they are not a fresh integration-test report.

---

## 1. Executive summary

1. **Phosphor natively runs Pi, rather than substituting its own agent.** For live sessions, it starts a Pi RPC process and normally consumes the user's existing `~/.pi/agent` configuration. The fact that a resource *loads* does not imply that its terminal-specific UI can display inside Phosphor.
2. **Phosphor injects six extensions into ordinary Pi sessions** using explicit command-line `-e` arguments. The bundle is hardcoded in the examined source. The documented GUI did not provide individual on/off switches for all six. One of them, Headroom, is functionally inert unless a specific proxy URL is enabled.
3. **The normal Pi `--no-extensions` flag does not suppress explicitly injected `-e` extensions.** Using it could disable the user's own discovered extensions while retaining the Phosphor bundle.
4. **Phosphor has exceptionally broad out-of-the-box rendering and artifact support:** KaTeX, Mermaid, Chart.js, Vega-Lite, sandboxed JavaScript-capable HTML, versioned artifacts, editor, terminal, diffs, worktrees and parallel agent workflows.
5. **A fully equivalent alternative with none of Phosphor's mandatory extension injection was not conclusively identified.** Several candidates offer meaningful subsets: `pi-kot`, `Pilot`, `pi-gui`, the `FaqFirebase/pi-desktop` project, PiChamber, Orbit and others. Their architecture, configuration reuse and feature completeness differ substantially.
6. **KittyBrowser (`graiz/kittybrowser`) can provide the key visual-rendering part directly inside Kitty.** It displays a real headless-Chrome page and therefore can run JavaScript, Canvas, interactive charts, mathematical diagrams and simulations. But it is **not** a Pi frontend, artifact version manager, file editor, Git review tool or workflow orchestrator.
7. **The most conservative architecture for the user's existing setup** is to keep the normal Pi coding agent and its extensions in one native Kitty pane; open KittyBrowser in another pane for live HTML/JavaScript previews; and use independent file/Git/editor tools for functions KittyBrowser does not provide. This avoids introducing Phosphor's six bundled extensions into the Pi process.

### The key distinction

- **Runtime:** Pi itself performs reasoning, tool calls, sessions, extension hooks, permission enforcement and configuration loading.
- **Presentation:** A frontend displays chats, rich Markdown, artifacts, editor panes, diffs and user interaction.
- **Browser engine:** Chromium renders and executes HTML/CSS/JavaScript, including Canvas and charts.
- **Artifact manager:** A separate mechanism creates, discovers, versions, stores and previews generated documents.

One product may include several of these layers, but support for one does not guarantee the others.

---

## 2. Relevant existing workflow and requirements

The discussion was motivated by a customized **Pi Coding Agent** configuration with these stated aims:

- Keep Pi's canonical agent configuration at **`~/.pi/agent`**, rather than relocating the entire configuration to a desktop frontend's proprietary directory.
- Preserve the existing Pi extensions, packages, skills, prompts, models, authentication, sessions and permission/sandbox decisions as much as possible.
- Maintain a **Kitty-first**, IDE-like layout with multiple terminal panes and separate TUI utilities.
- Present mathematical notation, diagrams and rich formatted answers; particularly **KaTeX**, **Mermaid**, **Chart.js** and **Vega-Lite**.
- Render **locally generated interactive HTML and JavaScript**, including double-pendulum/physics animations, graphs, controls, forms, dashboards and potentially Canvas/WebGL simulations.
- Support file browsing/editing, Git review/diffs and parallel or subagent workflows.
- Avoid mandatory duplicate extensions, unexpected tool registrations, unnecessary context/token overhead and configuration interference.
- Keep security and existing permission/sandbox guarantees under explicit user control.

The word **“bloatware”** was used to describe unwanted *automatically injected extensions*, not a verified performance finding that each extension is intrinsically wasteful. Each bundle component needs separate functional and token-cost scrutiny.

---

## 3. Phosphor: does it natively support Pi and use `~/.pi/agent`?

**Answer: yes, for the underlying Pi runtime and its ordinary resources; no, not for every terminal-specific UI behavior.**

### 3.1 Runtime integration

The examined Phosphor implementation starts actual Pi instances in RPC mode, commonly represented as:

```text
Phosphor desktop frontend
          |
          | Pi RPC messages over stdio
          v
     pi --mode rpc
          |
          v
    Pi runtime and resource discovery
          |
          v
       ~/.pi/agent
```

Key consequences:

- A Phosphor session uses Pi's models, providers, agent runtime and session semantics rather than an entirely different agent.
- Configuration is normally read from `~/.pi/agent`, or the directory selected by `PI_CODING_AGENT_DIR` if the process receives that environment variable.
- Workspaces can also have project-specific `.pi/` resources.
- Phosphor can add its own application preferences, directives, account overlays, frontend UI and supplementary extensions on top of standard Pi behavior.

Relevant source and documentation:

- [Phosphor — Pi integration](https://github.com/agustinsacco/phosphor/blob/main/docs/pi-integration.md)
- [Pi directory resolution](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/pi-paths.ts)
- [Phosphor session launcher](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/session-launch.ts)
- [Pi RPC client](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/rpc-client.ts)

### 3.2 Resource compatibility

| Resource / behavior | Expected compatibility in Phosphor | Caveat |
|---|---|---|
| `~/.pi/agent/settings.json` | Normal Pi loading | Spawn-time overrides or frontend settings can influence runtime behavior |
| `models.json`, `auth.json` | Normal Pi loading | Verify GUI launch environment and executable when installed through Nix/Home Manager |
| Pi-installed packages | Normal Pi loading | UI hooks may render differently or be unsupported in RPC |
| Skills and prompt templates | Normal Pi loading | Extensions/skills can be altered independently through the frontend |
| `AGENTS.md`, project `.pi/` | Normal Pi loading | Project trust and context-file toggles may matter |
| `mcp.json` | Via Pi runtime's MCP support | Verify server activation and environment details |
| JSONL sessions | Compatible with Pi's session mechanism | Frontend may add its own indexes, stores or session metadata |
| Pi terminal themes, custom footer/header/editor | **Not portable as-is** | Phosphor draws its own GUI |
| Custom terminal components / TUI controls | **Not guaranteed** | Pi RPC cannot automatically render all terminal UI APIs |

### 3.3 The major RPC extension-UI limitation

Pi's RPC extension-UI documentation makes an important distinction: an extension may be **loaded and executing** but its **terminal-specific visual component** may not appear. In the examined documentation, features such as custom terminal editor/header/footer/autocomplete components are not available in the ordinary terminal form over RPC; `ctx.ui.custom()` cannot be treated as a generally portable UI renderer.

For this user's configuration, compatibility requires individual testing of extensions such as rich-Markdown/LaTeX TUI renderers, custom UI panes, OpenTUI components and permission-dialog extensions. Nonvisual hooks or registered tools may still function even when their specialized terminal rendering does not.

Reference: [Pi RPC extension-UI compatibility](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/rpc-extension-ui.md).

### 3.4 Other Phosphor-specific behavior

- Phosphor draws its own desktop interface, color theme and shortcuts; these do not simply inherit Pi's Kitty terminal theme.
- It provides an **Advanced/health** inspection area for discovering the effective Pi executable/version and related runtime details.
- It may seed a bundled **`visual-artifacts` skill** into the user's Pi skills directory on first run if missing. The documented process does not overwrite an existing copy; the skill can be hidden or removed through the app's Skills UI, subject to its management rules.
- Phosphor's user-facing interface is **not** itself a sandbox on agent tool execution. Its docs state that Pi tools run with full permissions by default unless a permission gate is installed. Git worktree isolation is not a filesystem sandbox.
- Existing terminal launch-time safeguards and wrappers must be checked against the **actual process Phosphor launches**. In particular, a sandbox used in the Kitty shell does not automatically cover Pi subprocesses launched independently by a desktop app.

Reference: [Phosphor extensions and security](https://github.com/agustinsacco/phosphor/blob/main/docs/extensions.md).

---

## 4. Phosphor's six bundled extensions: confirmed source audit

### 4.1 The injection mechanism

The examined source declares these six files explicitly:

```ts
export const BUNDLED_EXTENSION_FILES = [
  'artifacts.ts',
  'context-breakdown.ts',
  'worktree-paths.ts',
  'tool-name-guard.ts',
  'headroom.ts',
  'lane-reload.ts',
] as const
```

They live in Phosphor's Pi-extension resources and are supplied to Pi with explicit `-e` arguments. The relevant loading path is approximately:

```text
bundledExtensions(resourceRoot)
    -> prepareSessionLaunch()
    -> prepareSessionPolicy()
    -> PiRpcClient.spawn()
    -> pi --mode rpc -e <file1> ... -e <file6>
```

In the examined source, this list is **hardcoded** rather than assembled from six normal user-facing enable switches. The tests for `prepareSessionLaunch()` explicitly expect all six extension paths.

Source files:

- [bundled-extensions.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/bundled-extensions.ts)
- [session-launch.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/session-launch.ts)
- [session-policy.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/session-policy.ts)
- [rpc-client.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/rpc-client.ts)
- [session-launch tests](https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/session-launch.test.ts)

### 4.2 What the six extensions actually do

| Bundled file | Purpose | Consequence of disabling it |
|---|---|---|
| **`artifacts.ts`** | Registers Phosphor's artifact tools; supports creating, editing, updating, reading and listing documents; integrates generated artifacts and session context | Phosphor's native artifact creation/version workflow would lose essential functionality unless replaced |
| **`context-breakdown.ts`** | Measures context composition and reports estimates/status to the frontend; also imports/registers other Phosphor features | Context-breakdown UI and imported features can be affected; it is more than a visual-only status bar |
| **`worktree-paths.ts`** | Detects a narrow class of mistaken cross-worktree file reads and blocks once with an explanation | Removes that additional protection against reading code from the wrong checkout; it is **not** a general sandbox |
| **`tool-name-guard.ts`** | Converts malformed tool-call names into safe text before bad names can poison persisted sessions | Removes a defensive repair against certain provider/tool-call errors |
| **`headroom.ts`** | Optionally compresses large eligible JSON tool results via a local Headroom proxy | Compression/status integration would be unavailable; **already inert** when `PHOSPHOR_HEADROOM_URL` is absent |
| **`lane-reload.ts`** | Registers a controlled `/phosphor-reload` mechanism to reload extensions/config between turns, including MCP changes | Can disrupt Phosphor's intended live config reload process |

Important details verified in source:

- `artifacts.ts` registers `artifact_help`, `artifact_create`, `artifact_edit`, `artifact_update`, `artifact_read` and `artifact_list`. Those tools add model-visible tool declarations and are not merely decorative. Source: [artifacts.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/artifacts.ts).
- `context-breakdown.ts` imports helpers for context budgets, swarm, PRs, context-file controls, global context and subagent-child handling. Calling this a completely passive meter would be misleading. Source: [context-breakdown.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/context-breakdown.ts).
- `worktree-paths.ts` applies a narrowly defined rule to certain Pi tool calls; it allows a repeated request as an explicit escape hatch. It is not equivalent to the user's separate sandbox/permission architecture. Source: [worktree-paths.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/worktree-paths.ts).
- `tool-name-guard.ts` listens to message completion and sanitizes malformed tool-call names, especially for provider-validation edge cases. Source: [tool-name-guard.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/tool-name-guard.ts).
- `headroom.ts` immediately returns without registering its tool-result hook if `PHOSPHOR_HEADROOM_URL` is unset. Thus **loaded** is not equivalent to **actively compressing content**. Source: [headroom.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/headroom.ts).
- `lane-reload.ts` registers the `phosphor-reload` command and contains safety checks for active work/subagents. Source: [lane-reload.ts](https://github.com/agustinsacco/phosphor/blob/main/libs/pi-extensions/pi-ext/lane-reload.ts).

### 4.3 What can be disabled without a fork?

| Item | Documented / confirmed mechanism | Status |
|---|---|---|
| User-installed Pi packages/extensions | Pi's normal package/resource settings and frontend management | **Yes** |
| Headroom compression behavior | Do not enable the managed Headroom service/proxy | **Yes**, although extension file is still passed |
| Optional Phosphor Brain | Disable/leave off in its own settings | **Yes**, optional functionality |
| Seeded `visual-artifacts` skill | Hide/remove from Skills (with documented persistence) | **Yes** |
| Each of six bundled `-e` files | No per-file toggle established in inspected documented settings | **Not through a verified setting** |

### 4.4 Why `pi --no-extensions` does **not** solve it

Pi's flag explicitly means **disable discovered, configured and built-in extensions**; it does **not** prevent explicit `-e <extension>` loading.

```sh
pi --no-extensions -e /some/explicit/extension.ts
```

The explicit extension still loads. Consequently, adding `--no-extensions` indiscriminately to a Phosphor-launched Pi process risks eliminating the **user's preferred extensions** while retaining Phosphor's injected six.

Source: [Pi CLI documentation](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/cli.md).

### 4.5 If full bundle control is required

The robust engineering approach is a **small focused fork** or an upstream change introducing an explicit, opt-in configuration field for the bundled file list, with defaults preserving Phosphor behavior. Merely deleting bundled resource files or manipulating paths in a packaged app is fragile and may cause startup/runtime errors.

A safe implementation should:

1. Make the list of Phosphor extensions explicitly configurable, ideally with **independent per-extension switches**.
2. Keep Pi's normal `~/.pi/agent` extension discovery intact.
3. Document feature dependencies — notably artifact tools, context-breakdown imports and reload logic.
4. Avoid silently disabling defensive guards without informed user choice.
5. Add tests proving the **actual argv** has exactly the selected `-e` files, and that user-configured Pi extensions still load.
6. Verify artifact creation, MCP reload, subagent activity, context UI, sessions and error handling after changes.
7. Verify security boundaries and permission gates in a production-like Linux/Kitty environment.

**Research conclusion:** There was no verified no-fork switch in the inspected Phosphor source and documented UI that disables arbitrary members of the six-extension bundle.

---

## 5. What makes Phosphor appealing: the requested feature set

The user highlighted this particular capability set as the benchmark for alternatives:

- **Rich Markdown and math:** KaTeX and highlighted code.
- **Diagrams:** Mermaid.
- **Charts:** Chart.js and Vega-Lite.
- **HTML/CSS/JavaScript:** model-authored HTML executed in a **script-enabled sandboxed iframe**, suitable for generated interactive controls and animations.
- **Artifacts:** a persistent artifact system with versioned previews, edit/update tools, comparisons and cross-session retrieval.
- **IDE utilities:** editable files, integrated terminal, Git diffs and review.
- **Multi-agent workflows:** parallel lanes and worktrees; integration with subagents.

### 5.1 Browser-rendering technology versus generated artifacts

The presence of a browser/WebView engine means HTML, scripts and chart libraries *can run* if the appropriate page is authored and permitted to load its dependencies. It does **not** automatically supply:

- Phosphor's in-chat diagram/chart fence recognition;
- its built-in KaTeX, Mermaid, Chart.js and Vega-Lite wiring;
- its artifact tool contracts;
- artifact version persistence or cross-session indexes;
- secure iframe policies matching Phosphor's;
- file editing, Git or agent orchestration.

### 5.2 Phosphor artifact isolation

The examined Phosphor docs describe HTML artifacts served via a dedicated `phosphor-artifact://` origin and presented in an iframe with `sandbox="allow-scripts"` **without** `allow-same-origin`. Network access is constrained by a separate restrictive content-security policy. This is a stronger *application-defined artifact isolation model* than simply opening an arbitrary HTML file in a normal Chrome browser.

Reference: [Phosphor extensions/artifact implementation](https://github.com/agustinsacco/phosphor/blob/main/docs/extensions.md).

---

## 6. Search for less intrusive Phosphor alternatives

We searched for Pi-native desktop/web frontends that could retain existing Pi configuration and approximate **all** the rendering, artifact, IDE and agent features **without automatically installing/injecting unwanted Pi extensions**.

### 6.1 Overall outcome

**No candidate was confirmed to satisfy the entire Phosphor benchmark with the same built-in artifact behavior and with a proven zero-extra-tool/extension footprint.** A project can be a better *fit* for one criterion yet worse on another.

| Candidate | Pi architecture / config fit | Rich interactive HTML | Editor / terminal / Git | Artifact versioning equivalent to Phosphor | Assessment |
|---|---|---|---|---|---|
| **[Phosphor](https://github.com/agustinsacco/phosphor)** | Native Pi RPC, standard config, but six injected extensions | **Verified documented** | **Yes** | **Yes** | Functional benchmark; unwanted injection |
| **[pi-kot](https://github.com/keemzin/pi-kot)** | Pi SDK; documents `PI_CONFIG_DIR` defaulting to `~/.pi/agent` | **Yes**, sandboxed inline HTML | **Yes**, according to README | **Not established as equivalent** | Strong browser-based contender; native extra agent behaviors need auditing |
| **[Pilot](https://github.com/espennilsen/pilot)** | Pi SDK, but separate Pilot config directory by default | **Yes**, live HTML/CSS/JS | **Yes** | **Unclear/incomplete** | More intrusive relative to existing config and tools |
| **[pi-gui](https://github.com/minghinmatthewlam/pi-gui)** | Thin Pi SDK driver and Pi sessions/settings | Extension-view iframes documented; **general generated-artifact parity unproven** | **Yes** | **Not established** | Good lightweight IDE shell, not yet verified visual-artifact equivalent |
| **[FaqFirebase/pi-desktop](https://github.com/FaqFirebase/pi-desktop)** | Launches Pi RPC, supports `~/.pi/agent` models/sessions | HTML file preview documented; arbitrary interactive artifact semantics unverified | **Yes** | **Not established** | Comprehensive IDE, but renderer/artifact gaps remain |
| **[AMagicPear/pichamber](https://github.com/AMagicPear/pichamber)** | Pi workspace shell, Bun + Vue | Web rendering and KaTeX/Mermaid dependencies documented; full HTML artifact parity unverified | Shell terminal; other coverage must be assessed | **Not established** | Interesting browser UI, but feature-by-feature proof insufficient |
| **[ryderasking/pichamber](https://github.com/ryderasking/pichamber)** | Distinct PiChamber codebase with Pi extension integration | **Yes for extension-defined sandboxed app surfaces** | Varies by app surface/version | **Not established** | More advanced extension UI model, not automatically Phosphor parity |
| **[Orbit](https://github.com/imrj05/orbit)** | Native Pi RPC; strong config/session reuse | **No integrated browser engine**; limited native Mermaid diagrams | **Yes** | **Not established** | Strong native IDE, weak for arbitrary HTML/JS visualization |
| **[pi-dashboard](https://github.com/samfoy/pi-dashboard)** | Pi RPC browser/iOS dashboard | HTML document preview direction; full support uncertain | Files, Git-related UI, terminal | **Not established** | Useful remote dashboard, not proven exact replacement |
| **[Pithagoras](https://github.com/thecodacus/pithagoras)** | Pi SDK-backed web portal with separate portal management | Canvas and optional features, but no verified Phosphor-equivalent renderers | Project files, Git, terminal, workflows | **Not established** | Broad multi-agent portal, potentially more services/configuration |

**Interpretation:** “Not established” is deliberately **not** the same as “definitely absent.” It means the inspected docs/source did not establish exact parity, and no hands-on proof was produced.

### 6.2 pi-kot: important details

Repository: [keemzin/pi-kot](https://github.com/keemzin/pi-kot).

Documented capabilities include:

- Streaming chat, formatted code, KaTeX and inline Mermaid.
- HTML/SVG previews in sandboxed iframes; generated files under a project's `.pi/artifacts/` can be opened in an artifact viewer.
- A React/TypeScript web frontend with an editor (CodeMirror), file explorer, Git views, terminal support and subagent-related views.
- Default `PI_CONFIG_DIR` documented as `~/.pi/agent`, although pi-kot also maintains its own UI/session and service-related state separately.
- Built-in plan mode, tool orchestration/Codemode and other agent-environment features, some of which can change the model's tool surface or execution behavior.

**Risks/remaining checks:** exact artifact revision tracking, compatibility with the user's particular permission-mode/sandbox extensions, whether default plan/Codemode tools conflict with the existing setup, and full standalone Vega-Lite/Chart.js support. Basic browser ability to display an HTML file with those libraries is not the same as native support for their code fences.

Relevant files:

- [README and configuration](https://github.com/keemzin/pi-kot/blob/master/README.md)
- [Artifact viewer](https://github.com/keemzin/pi-kot/blob/master/packages/client/src/components/ArtifactViewer.tsx)
- [Artifact API](https://github.com/keemzin/pi-kot/blob/master/packages/server/src/routes/artifacts.ts)
- [Web UI context instructions](https://github.com/keemzin/pi-kot/blob/master/packages/server/src/contexts/web-ui.md)

### 6.3 Pilot: important details

Repository: [espennilsen/pilot](https://github.com/espennilsen/pilot).

Documented strengths include HTML/CSS/JavaScript live preview, file editor, Git tools, subagents, terminal and multiple workspaces.

However, its documented Linux default configuration is **`$XDG_CONFIG_HOME/pilot`**, usually `~/.config/pilot/`; its `piAgentDir` setting defaults to that Pilot directory. Pilot also has separate SDK-level/project settings, MCP management, memory, prompt/task tooling and other services. Hence, it should **not** be assumed to automatically adopt all resources and security assumptions from `~/.pi/agent` without configuration and testing.

The source contains an artifact UI/store, but a project `docs/GAPS.md` note described incomplete “true artifact system” support. That discrepancy warrants caution before claiming Phosphor-equivalent persistent/versioned artifacts.

Sources: [Pilot README](https://github.com/espennilsen/pilot/blob/main/README.md), [Pilot configuration](https://github.com/espennilsen/pilot/blob/main/docs/CONFIGURATION.md), [Pilot gaps](https://github.com/espennilsen/pilot/blob/main/docs/GAPS.md).

### 6.4 pi-gui: important details

Repository: [minghinmatthewlam/pi-gui](https://github.com/minghinmatthewlam/pi-gui).

- Designed as a desktop shell around the Pi runtime, using Pi's settings, sessions and credentials.
- Has parallel threads/worktrees, file explorer/editor, terminal, Git diff/review and extension management.
- Its documented desktop-extension architecture provides an **opaque-origin `sandbox="allow-scripts"` iframe** for extension-authored frontend assets.
- But an iframe for an *extension's own GUI panel* is **not evidence** that arbitrary model-generated HTML, KaTeX/Mermaid/Chart.js/Vega-Lite code blocks and a Phosphor-like artifact version store all work natively.

Sources: [pi-gui README](https://github.com/minghinmatthewlam/pi-gui/blob/main/README.md), [desktop extension architecture](https://github.com/minghinmatthewlam/pi-gui/blob/main/docs/architecture.md).

### 6.5 Orbit: important details

Repository: [imrj05/orbit](https://github.com/imrj05/orbit).

- Rust/GPUI native desktop workbench.
- Talks to the real Pi RPC executable, using the same Pi sessions and configuration approach.
- Provides editor, Git/worktrees, review, terminal and a native, limited Mermaid renderer.
- **No browser/WebView engine** is a deliberate architectural characteristic. Arbitrary HTML/JavaScript simulation previews therefore are **not** a native drop-in feature.
- The examined docs also acknowledge some extension-management work remains in progress, and Orbit itself has bundled Pi-side support components that would need inspection if zero injection is a hard requirement.

Source: [Orbit README](https://github.com/imrj05/orbit/blob/main/README.md).

### 6.6 The two distinct PiChamber repositories

Research surfaced **two different codebases** with the same general name, so their features must **not** be combined into one imagined product:

- [AMagicPear/pichamber](https://github.com/AMagicPear/pichamber) — Vue 3 + Bun browser workspace, terminal and frontend rendering dependencies including KaTeX/Mermaid.
- [ryderasking/pichamber](https://github.com/ryderasking/pichamber) — a different PiChamber project documented by Context7, including an extension UI mechanism where a Pi extension can create a sandboxed script-enabled app surface by appending a `pichamber.app` entry.

The second project's documented interface uses small declarative/HTML UI protocols and session-scoped app surfaces, with explicit caps and restrictive sandboxing. This is valuable but **not** proof that every model response automatically renders as an interactive, persistent artifact.

References: [AMagicPear repo](https://github.com/AMagicPear/pichamber), [ryderasking extension docs](https://github.com/ryderasking/pichamber/blob/main/packages/docs/content/docs/extensions.mdx).

### 6.7 Other related visual-artifact projects

- [lleontor705/pi-canvas](https://github.com/lleontor705/pi-canvas) — optional Pi extension exposing a locally hosted artifact canvas with Mermaid, SVG, sandboxed HTML and live updates. It is an **artifact viewer**, not a complete IDE frontend.
- [jakeryderv/pi-artifacts](https://github.com/jakeryderv/pi-artifacts) — optional Pi artifact authoring/rendering package with KaTeX/Markdown, Mermaid, Chart.js and portable exports; its documented HTML stack emphasizes **declarative components rather than arbitrary authored JavaScript**.
- [neriousy/opentui-browser](https://github.com/neriousy/opentui-browser) — experimental embedded browser rendering for OpenTUI through Kitty graphics, not a general Pi GUI replacement; documentation examined earlier showed an older supported OpenTUI version range, so it should not be assumed to work with newer OpenTUI frontends without porting.
- [konaito/terminal-browser](https://github.com/konaito/terminal-browser), [zenbu-labs/terminal-browser](https://github.com/zenbu-labs/terminal-browser) and [sanohiro/casty](https://github.com/sanohiro/casty) — alternative real-browser-in-terminal projects. Their runtime, platform support, pane integration and security characteristics differ from KittyBrowser.

---

## 7. KittyBrowser (`graiz/kittybrowser`): what it actually does

Repository: **[graiz/kittybrowser](https://github.com/graiz/kittybrowser)**. Binary: **`kb`**.

KittyBrowser is a Rust application that controls **headless Chrome/Chromium** using the Chrome DevTools Protocol and paints each browser frame into a terminal using the **Kitty graphics protocol**. It accepts keyboard/mouse interaction and exposes an automation/control interface (`kb ctl`), an optional MCP interface and a reusable agent skill.

This is a *full graphical browser engine in a terminal pane*, **not** a text-only renderer such as Lynx and **not** a native Pi extension.

### 7.1 Capabilities relevant to the benchmark

| Capability | KittyBrowser answer | Explanation |
|---|---|---|
| Modern HTML/CSS rendering | **Yes** | Rendered by actual Chrome |
| JavaScript execution | **Yes** | Chrome executes the web page's scripts |
| Dynamic CSS animations and SVG | **Yes** | Standard browser functionality |
| Interactive 2D physics simulations | **Yes, in principle** | The generated HTML/JavaScript must implement the simulation |
| Canvas drawing/animation | **Yes** | Page Canvas engine runs in Chrome |
| WebGL/3D simulation | **Potentially** | Subject to local Chrome GPU/software-rendering availability and performance |
| KaTeX math | **Yes, if the page includes KaTeX** | Not an automatic formatter for Pi output |
| Mermaid diagrams | **Yes, if the page includes Mermaid** | Not automatic recognition of all Pi fenced code |
| Chart.js | **Yes, if the page includes Chart.js** | Page supplies chart data and library |
| Vega-Lite | **Yes, if the page includes Vega/Vega-Lite** | Page supplies spec and renderer |
| Buttons/sliders/forms/pointer events | **Yes** | Browser interactions work using mouse/keyboard |
| Interactive artifact version history | **No built-in equivalent** | Separate artifact storage/version system required |
| Pi chat UI or agent orchestration | **No** | Pi remains in another pane/process |
| Source-code editor / file browser | **No built-in IDE** | Use a dedicated editor/TUI or GUI |
| Git diffs/review | **No** | Use separate Git tooling |
| Pi permission/sandbox enforcement | **No** | Use existing Pi-side security controls |

**Clarification:** JavaScript-capable Chromium lets the browser *run* a double-pendulum simulation after Pi creates a valid HTML/JS file. It does not cause Pi to generate, test or version that simulation automatically.

### 7.2 Working with local artifacts

The documented command accepts both ordinary URLs and local files:

```sh
# Open a locally generated HTML page in the current terminal pane.
kb ./simulation.html

# Open a local preview/dev server.
kb http://localhost:5173
```

Examples of browser-side validation:

```sh
kb ctl check
kb ctl console --errors
kb ctl screenshot --out preview.png
kb ctl viewport 1280x800
```

Other documented commands permit opening and reloading a page, mouse clicks, typing, selecting, scrolling, retrieving visible text or HTML, capturing screenshots, inspecting console/network errors, running JavaScript with `eval`, and switching viewport widths. **These commands target a running KittyBrowser**, not Pi itself.

To have an agent control the correct browser when several instances exist, select the instance by its ID using the `-i` option. Without it, control can address a different running instance.

Reference: [KittyBrowser README](https://github.com/graiz/kittybrowser/blob/main/README.md), [agent skill](https://github.com/graiz/kittybrowser/blob/main/skill/SKILL.md).

### 7.3 Example Kitty layout

```text
+---------------------------------------------------------------------+
|                           KITTY WINDOW                              |
+--------------------------------+------------------------------------+
| Pi Coding Agent                | KittyBrowser (`kb`)                |
|                                |                                    |
| ~/.pi/agent config             | Real HTML/CSS/JS live preview      |
| Original skills and extensions | Canvas / SVG / KaTeX / Mermaid     |
| Existing permission rules      | Charts / buttons / simulations     |
| Subagent workflows             |                                    |
|                                | `kb ./simulation.html`             |
+--------------------------------+------------------------------------+
| Optional additional native Kitty split: editor, file tree or Git TUI |
+---------------------------------------------------------------------+
```

The suggested workflow is:

1. Start normal Pi in a Kitty pane using the user's established launch/sandbox workflow.
2. Ask Pi to author `simulation.html` or a small web project; retain the source in the workspace.
3. Start `kb ./simulation.html` or visit the local dev server in another **native Kitty split**.
4. Interact visually with the page while Pi edits source files.
5. Run `kb ctl check` and other targeted checks, then independently test the simulation behavior and physics.
6. Keep any desired artifact version history using Git or a separate opt-in artifact tool; KittyBrowser does not supply it.

This architecture preserves the **original Pi extension loader and TUI rendering**, because Pi itself continues in its native terminal rather than an RPC frontend.

### 7.4 KittyBrowser requirements and maturity

At the time of the research:

- Requires a terminal that supports the **Kitty graphics protocol**.
- Requires Chrome, Chromium, Edge or Brave (or `KB_CHROME` pointing to a valid browser executable).
- Requires Rust to build from source; the README reported no prebuilt releases yet.
- On Linux, the documented starting point is:

```sh
cargo install --git https://github.com/graiz/kittybrowser
kb doctor
kb ./simulation.html
```

- The author characterized the software as **early but usable, v0.1**. The documentation explicitly said Linux and Kitty combinations were not yet thoroughly tested, though compilation worked.
- **tmux, screen and Zellij are unsupported** in its documented rendering approach. Prefer **Kitty's own windows/splits**, not a terminal multiplexer wrapping the browser pane.
- Browser frame capture and terminal image transmission introduce overhead. CPU usage, latency, video smoothness and WebGL behavior should be tested on the actual Linux hardware. Smooth 60-fps rendering cannot be guaranteed.
- The roadmap lists further performance and editing-feedback work; these should not be assumed to have shipped.

Sources: [README](https://github.com/graiz/kittybrowser/blob/main/README.md), [roadmap](https://github.com/graiz/kittybrowser/blob/main/docs/ROADMAP.md), [architecture](https://github.com/graiz/kittybrowser/blob/main/docs/ARCHITECTURE.md).

### 7.5 Security differences from Phosphor artifacts

**KittyBrowser is a normal browser; it is not a replica of Phosphor's restricted artifact iframe.**

- Chrome provides its ordinary browser sandbox and web security model.
- Local model-generated HTML can run scripts and may attempt network requests or interactions allowed by the browser. Treat untrusted HTML as **active content** rather than inert output.
- KittyBrowser has its own cookie/profile directory, separate from everyday Chrome.
- Its Unix control socket is restricted to the local user, but programs with access to that socket can control the active browser and run JavaScript in it.
- Its optional MCP interface is off until enabled.
- Optional remote DevTools access (started with `--devtools`) exposes a localhost debugging port; the project's security documentation warns about local access to that port.
- The browser control surface is **not** a substitute for a Pi filesystem sandbox or tool-permission gate.

Source: [KittyBrowser security policy](https://github.com/graiz/kittybrowser/blob/main/.github/SECURITY.md).

---

## 8. Comparing the complete solutions by architecture

### Option A — Keep Phosphor as-is

**Pros:** Most complete integrated experience; built-in versioned artifacts; full desktop IDE features; first-class HTML/JavaScript previews and rich Markdown.

**Cons:** Six mandatory `-e` extensions in ordinary sessions; its own UI differs from native Kitty Pi; terminal-specific extensions may fail to render; additional process-launch/security checks required.

**Best when:** An integrated desktop experience outweighs the desire for minimal Pi process modification.

### Option B — Use a focused Phosphor fork

**Pros:** Retains Phosphor's built-in artifact ecosystem and integrated UI, while allowing an extension policy explicitly chosen by the user.

**Cons:** Requires maintaining and testing a fork; features may depend on bundled extensions; disabling artifact tools without replacement will break artifact creation; some protections will be removed if their guard files are disabled.

**Best when:** Phosphor's artifact/versioning interface is essential, but automatic extension injection is unacceptable.

### Option C — Adopt a different Pi GUI (for example pi-kot or pi-gui)

**Pros:** Potentially fewer mandatory extensions or a more direct use of Pi settings; strong IDE/browser features depending on product.

**Cons:** None was fully verified to match all of Phosphor's renderer, persistent-artifact, extension and sandbox capabilities. Some have their own in-process tools, contexts, agents or data directories that could create different, equally unwanted overhead.

**Best when:** A specific alternative passes a hands-on compatibility and token-surface audit against the user's config.

### Option D — Native Pi + KittyBrowser + separate TUI tools

**Pros:** Keeps normal Pi launch and `~/.pi/agent` resources; no Phosphor-specific extensions; full real-browser rendering in an adjacent Kitty pane; modular and opt-in; no need to replace normal native TUI extensions.

**Cons:** Not one integrated app; no automatic artifact version store; browser rendering is separate from the Pi transcript; early Linux/Kitty maturity and resource overhead need verification.

**Best when:** The core goal is **interactive HTML/JS visualizations within a Kitty-first environment with minimal changes to Pi itself**.

### Decision at the end of this discussion

**Provisional recommendation: Option D** for the user's present priorities — **retain normal Pi in Kitty, and test KittyBrowser as the browser renderer**. It is a recommendation about architecture, **not** a claim that KittyBrowser replaces Phosphor's artifact database, editor or workflow UI.

If the requirement is instead **every Phosphor feature in one window**, then **Option B (a narrowly customized Phosphor build)** remains the clearer design path, subject to rigorous dependency and compatibility validation.

---

## 9. Concrete acceptance tests before committing to any solution

The conversation identified the need to test on the user's real Linux/Kitty environment instead of assuming compatibility from README claims.

### 9.1 Verify Pi identity and configuration

```sh
command -v pi
pi --version
printf 'PI_CODING_AGENT_DIR=%s\n' "${PI_CODING_AGENT_DIR:-$HOME/.pi/agent}"
```

Confirm whether the GUI/terminal uses the **same Pi executable, environment and effective agent directory**. On a Nix/Home Manager setup, desktop-launched `PATH` and interactive-shell `PATH` are not necessarily identical.

### 9.2 Verify extension/tool overhead

- Enumerate actual extensions discovered by Pi and any extensions separately injected by the frontend.
- Record the process launch arguments to confirm what `-e` paths load; do not mistake a settings toggle for proof.
- Compare active tool names and model-visible tool schemas before/after frontend changes.
- Test whether the existing permission-mode UI and sandbox wrapper still enforce the intended restrictions.
- Check for overlap among built-in workflow/planning/context extensions, existing subagent tools and any Pi extension that modifies model context.
- Measure added context and latency using a repeatable baseline; do not infer token costs solely from executable size.

### 9.3 Verify rich content

Use a small fixed acceptance set:

1. Inline and display **KaTeX**, including nontrivial LaTeX.
2. A **Mermaid** flowchart and sequence diagram.
3. **Chart.js** line chart with hover tooltips and togglable series.
4. **Vega-Lite** visualization with interaction/filtering.
5. A self-contained HTML file with working buttons, sliders and dynamically updated text.
6. A **double-pendulum** simulation using `requestAnimationFrame` and Canvas; demonstrate start/pause/reset controls.
7. Optional WebGL scene (report support honestly if Chrome GPU initialization fails).
8. Responsive layout at desktop and mobile viewports.
9. Console/network error checks and screenshots, including after a source edit/reload.

### 9.4 Verify artifact and workflow features separately

- Create an artifact, revise it twice, compare versions, restart the frontend and resume the session.
- Check whether artifact history survives context compaction or session deletion; distinguish filesystem/Git history from a dedicated artifact library.
- Edit a file in the UI, inspect Git diffs, stage/discard changes and verify the real filesystem state.
- Start two tasks in separate worktrees and ensure there is no accidental cross-worktree reading/writing.
- Run a subagent workflow and verify dialog/approval handling without bypassing existing user authorization rules.
- Confirm that no new unapproved network services, MCP tools or persistent background processes are installed.

### 9.5 Important tests specific to KittyBrowser

- Run `kb doctor` in a **native Kitty pane**.
- Open a local HTML/JS simulation and confirm pointer/keyboard input works.
- Check high-DPI scaling, resizing, smoothness, CPU/RAM use and animation responsiveness on actual hardware.
- Verify that KittyBrowser works **outside** tmux/screen/Zellij.
- Test the desired browser integration method (manual CLI, reusable skill or optional MCP) without silently adding a large always-on tool schema.
- Review browser profiles and local socket access before enabling agent control.

---

## 10. Items left unresolved

The discussion did **not** establish any of the following as completed or definitively solved:

1. A production-tested, extension-free replacement for **all** Phosphor functionality.
2. An official Phosphor setting that disables arbitrary members of its six hardcoded injected extensions.
3. Exact performance/token comparisons of Phosphor, pi-kot, Pilot, pi-gui and other frontends on the user's machine.
4. Full compatibility of every current Pi extension and customized permission/sandbox mechanism with any RPC-based frontend.
5. A Linux/Kitty real-hardware benchmark of KittyBrowser physics simulations or WebGL acceleration.
6. An automatic, no-extra-tool integration that turns any Pi response into a persistent, versioned, interactive KittyBrowser artifact.
7. Proof that `pi-kot`/`pi-gui`/PiChamber fully matches Phosphor's KaTeX + Mermaid + Chart.js + Vega-Lite + arbitrary JavaScript + persistent artifact store as a single native feature set.
8. User approval for installing/forking any product or modifying the existing `~/.pi/agent` configuration.

These are meaningful engineering follow-ups, not reasons to discount capabilities that were positively verified.

---

## 11. Reference links

### Phosphor and Pi

- Phosphor repository: https://github.com/agustinsacco/phosphor
- Phosphor Pi integration: https://github.com/agustinsacco/phosphor/blob/main/docs/pi-integration.md
- Phosphor bundled extensions and artifact security: https://github.com/agustinsacco/phosphor/blob/main/docs/extensions.md
- Phosphor launch extension list: https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/bundled-extensions.ts
- Phosphor RPC process spawn: https://github.com/agustinsacco/phosphor/blob/main/libs/session-runtime/src/pi/rpc-client.ts
- Pi RPC extension UI: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/rpc-extension-ui.md
- Pi CLI flags: https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/cli.md

### Alternative Pi frontends and artifact tooling

- pi-kot: https://github.com/keemzin/pi-kot
- Pilot: https://github.com/espennilsen/pilot
- pi-gui: https://github.com/minghinmatthewlam/pi-gui
- Pi Desktop: https://github.com/FaqFirebase/pi-desktop
- PiChamber (AMagicPear): https://github.com/AMagicPear/pichamber
- PiChamber (ryderasking): https://github.com/ryderasking/pichamber
- Orbit: https://github.com/imrj05/orbit
- Pithagoras: https://github.com/thecodacus/pithagoras
- pi-dashboard: https://github.com/samfoy/pi-dashboard
- pi-canvas: https://github.com/lleontor705/pi-canvas
- pi-artifacts: https://github.com/jakeryderv/pi-artifacts

### Real browser in terminal

- KittyBrowser (`kb`): https://github.com/graiz/kittybrowser
- KittyBrowser architecture: https://github.com/graiz/kittybrowser/blob/main/docs/ARCHITECTURE.md
- KittyBrowser roadmap: https://github.com/graiz/kittybrowser/blob/main/docs/ROADMAP.md
- KittyBrowser security: https://github.com/graiz/kittybrowser/blob/main/.github/SECURITY.md
- KittyBrowser skill: https://github.com/graiz/kittybrowser/blob/main/skill/SKILL.md
- terminal-browser (konaito): https://github.com/konaito/terminal-browser
- terminal-browser (zenbu-labs): https://github.com/zenbu-labs/terminal-browser
- casty: https://github.com/sanohiro/casty
- opentui-browser: https://github.com/neriousy/opentui-browser

---

## 12. Bottom line

> **If preserving the current Pi configuration and keeping unwanted extensions out of the model/runtime is the higher priority, use native Pi in Kitty and experiment with KittyBrowser as a separate interactive-browser pane.** This gets real JavaScript/Canvas/charts without requiring Phosphor's additional Pi extension bundle, but it is a modular workflow rather than a full desktop artifact IDE.
>
> **If one unified desktop interface with persistent, versioned artifacts is nonnegotiable, Phosphor remains the strongest verified match discussed here.** A targeted, well-tested configuration/fork to make its own bundled extensions selectable is more defensible than assuming another frontend already has complete parity.

**End of session summary.**
