# Sources and verification ledger — Pi-maintenance Codex rewrite

**Researched:** 19 November 2026. This is the **only** artifact containing research links and attribution. The Codex task prompt, Codex `AGENTS.md`, advisor workflow, discovery tests and TOML files are deliberately citation-free. This ledger distinguishes (a) the user's requirements, (b) inspected archived evidence, (c) official/source-verified behavior, and (d) design choices or limitations.

## 1. User requirements: primary specification, not inferred facts

| ID | Direct user requirement | Implemented in |
|---|---|---|
| U1 | Reorganize the *live* `~/.pi/agent/`, keeping `auth.json` and `settings.json` at root; relocate every other JSON **where technically safe**. JSON may originate from Pi or extensions. | Main prompt, §1 |
| U2 | Organize custom code under `extensions/`, update **all** path consumers, ensure Pi and installed extensions work with new locations. | Main prompt, §§2–3 |
| U3 | Fully audit Pi installation; read every Markdown; inspect actual code/config independently of documentation and fix verified defects. | Main prompt, §4 |
| U4 | Codex can perform only allowed noninteractive checks; no running Pi/SDK in sandbox, no bwrap, no interactive Pi tests; produce full user-only Pi tests under `~/.config/pi/Tests/` and a separate final automated validation. | Main prompt, Tests |
| U5 | Preserve Codex's global `~/.codex/AGENTS.md`; use Codex-specific `~/.pi/AGENTS.md`; Pi's `~/.pi/agent/AGENTS.md` belongs to Pi. Option A invocation suppresses project instruction injection. | `README-INSTALL.md`, Codex `AGENTS.md`, task prompt's single initial directive |
| U6 | GPT-6.1 **Sol** is the worker/default; **GPT-6 Astra (`gpt-6-astra`)** is a **required project-only advisor when Sol gets stuck**, not a generic remedy for blocked actions. | Codex `AGENTS.md`, project `.codex/config.toml`, project `pi_astra_advisor.toml`, advisor workflow |
| U7 | No global Pi-specific advisor config; advisor-related files belong somewhere under `~/.pi` or `~/.pi/agent`; test **Codex detecting the advisor**, not model execution or switching. | `.codex/` placement and `tests/ADVISOR_DISCOVERY.md` |
| U8 | Rework all relevant Markdown/config files from scratch; remove invented instructions; source all technical statements separately without citations inside operational prompts. | All deliverables and this ledger |

The user expressly corrected **“stuck” versus “blocked”**. The distinction is a user requirement, not an extension API. Whether the model will consistently detect stuckness is an agent behavior to observe, not a guaranteed Codex automation hook.

## 2. Local source inspected (not proof of live contents)

`/mnt/data/pi-config-reconstructed.zip` was opened as a **historical snapshot**; the user said its contents had been copied to `~/.pi/agent/`. Relevant observed archived paths:

- `agent/settings.json`, `agent/auth.json` not listed in archive (credentials may be excluded), `agent/AGENTS.md`, `agent/mcp.json`, `agent/models.json`, `agent/keybindings.json`, `agent/sandbox.json`, `agent/models-store.json`, `agent/mcp-on-demand.json`.
- `agent/extensions/pi-mcp-on-demand/`, `agent/pi-mcp-on-demand/`, `agent/extensions/prompt-review-guard.ts`, `agent/prompt-review/`, `agent/workflow/` and `agent/agents/`.
- `agent/pi-subagents/last-seen-version.json`, `agent/permission-mode/state.json`, `agent/pi-extensions.txt`, `agent/npm/` and `agent/install/`.

The snapshot gives **candidate paths** to trace, not proof of their current ownership, installed versions, active loading state, or relocation support. Code must inspect actual live files before changing them. The prior task drafts were also read, but are not treated as technical evidence.

## 3. Upstream and Codex documentation (checked via Keenable; Context7 cross-checks noted)

| ID | Link / research tool | Exact supported fact and where applied |
|---|---|---|
| P1 | [Pi configuration, version 1.5.2](https://raw.githubusercontent.com/earendil-works/pi/v1.5.2/packages/coding-agent/docs/configuration.md) — Keenable | Pi's agent directory has documented conventional paths for `settings.json`, `auth.json`, `mcp.json`, `models.json`, `keybindings.json`, `AGENTS.md`, extensions, skills, prompts and themes; context files may include ancestor instruction files. Underpins file exceptions, Pi/ancestor AGENTS caveat. |
| P2 | [Pi settings, version 1.5.2](https://raw.githubusercontent.com/earendil-works/pi/v1.5.2/packages/coding-agent/docs/settings.md) — Keenable | Explicit settings resources include `packages`, `extensions`, `skills`, `prompts`, `themes`. No supported universal redirect for arbitrary JSON follows from this. Underpins no invented `settings.json` keys. |
| P3 | [Pi extensions, version 1.5.2](https://raw.githubusercontent.com/earendil-works/pi/v1.5.2/packages/coding-agent/docs/extensions.md) — Keenable | Auto-discovery of `extensions/*.ts|js`, one-level `extensions/*/index.ts|js`, or manifest-declared entry points; explicit extra paths via settings. Underpins safe relocation of code. |
| P4 | [pi-subagents configuration](https://github.com/nicobailon/pi-subagents/blob/main/docs/configuration.md) — Keenable | Example installed third-party extension uses fixed user config at `~/.pi/agent/extensions/subagent/config.json`, while certain settings live in Pi's `settings.json`; illustrates need to inspect extension-owned readers/writers. |
| P5 | [pi-permission-modes v2.7.2 configuration](https://cdn.jsdelivr.net/npm/pi-permission-modes@2.7.2/docs/configuration.md) — Keenable | Extension-specific global path `~/.pi/agent/permission-mode/permission-mode.json`; `state.json` tracks last-seen version for default-change notices; missing config falls back to stock. Illustrates generated state and path exceptions. |
| C1 | [Codex custom AGENTS.md discovery](https://developers.openai.com/codex/guides/agents-md) — Keenable | Global Codex file under `$CODEX_HOME` (usually `~/.codex/`) is distinct from project files; `AGENTS.override.md` precedence, project root→cwd discovery. Underpins instruction separation. |
| C2 | [Codex 0.165.0 project instruction source](https://github.com/openai/codex/blob/rust-v0.165.0/codex-rs/core/src/agents_md.rs) — Keenable and prior source inspection | `project_doc_max_bytes=0` turns off project AGENTS injection while global instructions are assembled separately. It is not a filesystem access-control switch. Underpins launch guidance. |
| C3 | [Codex CLI reference](https://developers.openai.com/codex/cli/reference) — Keenable | `-C/--cd` chooses working directory, `-m` chooses model, `-c` sets config override; `/status` shows current session model and workspace. Underpins README launch. |
| C4 | [Codex advanced configuration](https://developers.openai.com/codex/config-file/config-advanced) — Keenable | `.codex/config.toml` is a project-scoped layer and trusted projects govern loading; `.git` is default root marker, changeable via `project_root_markers`; missing root and trust can affect discovery. Underpins local configuration, B discovery test. |
| C5 | [Codex subagents](https://developers.openai.com/codex/subagents) — Keenable | Native personal custom-agent location `~/.codex/agents/` **versus project-scoped** `<project>/.codex/agents/`; required `name`, `description`, `developer_instructions`; `model`, `model_reasoning_effort` supported; `[agents].enabled`; subagents inherit parent permissions; feature restrictions are available. Underpins `.codex/` TOML and workflow. |
| C6 | [Codex model selection](https://developers.openai.com/codex/models) — Keenable | The precise documented CLI IDs are `gpt-6.1-sol` and **`gpt-6-astra`**. No automatic `gpt-6.2-astra` substitution is requested. Underpins project model and advisor definition. |
| C7 | [OpenAI GPT-6 Astra model spec](https://developers.openai.com/api/docs/models/gpt-6-astra) — Keenable/web verification | `gpt-6-astra` supports reasoning effort `medium` (also low/high/xhigh/max); used in custom agent TOML. Availability depends on account/model access and cannot be certified here. |
| C8 | [Codex CLI developer commands](https://developers.openai.com/codex/developer-commands) — Keenable | `/agents`/`codex agents` are for a **chat command center**, `/subagents` switches among active agent threads, `/status` is session status. These are not guaranteed registries of installed custom-agent definitions. Underpins discovery-test caution. |
| C9 | [Codex developer settings](https://developers.openai.com/codex/developer-settings) — Keenable | `/debug-config` exposes effective configuration layers when available. Proves project layer only, not loaded role type. |
| C10 | [Codex permissions](https://developers.openai.com/codex/permissions) — Keenable | Workspace access/approval and protected `.codex/` paths are independent from AGENTS discovery; parent permissions apply to subagents. Underpins install-outside-Codex and warning against treating instructions as a sandbox. |
| C11 | [Codex best practices](https://developers.openai.com/codex/learn/best-practices) and [Codex prompting](https://developers.openai.com/codex/prompting) — Keenable | Prompts should state goal/context/constraints/done; keep AGENTS concise, use source-based changes, verify results, avoid speculative or unrelated modifications. Underpins compact separation of task versus durable policy. |
| C12 | [Codex GPT-6.2 prompting best practices](https://developers.openai.com/codex/learn/prompting-gpt-6-2-astra) — Keenable | Keep AGENTS lean and use subagents for well-scoped special work. Applied as **writing style**; not permission to choose GPT-6.2. |
| C13 | [Codex project config precedence (rust-v0.165.0)](https://github.com/openai/codex/blob/rust-v0.165.0/codex-rs/config/src/loader/README.md) — Context7 `/openai/codex/rust-v0.165.0` | CLI session flags precede project config; project scope and trust remain distinct from AGENTS injection. Underpins invocation plus local default model. |
| T1 | [Python `tomllib` documentation](https://docs.python.org/3/library/tomllib.html) — Keenable | `tomllib` is part of Python 3.11+; used in the static TOML parse stage of advisor discovery tests. |
| C14 | [Pi Context7 documentation](https://github.com/earendil-works/pi/blob/v1.4.0/packages/coding-agent/docs/configuration.md) — Context7 `/earendil-works/pi/v1.4.0` | Independent check of canonical Pi agent paths, contexts and SDK directory responsibilities. Version 1.4.0 is corroborating context only, not a substitute for installed 1.5.2 docs. |

## 4. Known limitations and contradictory/older evidence

1. **Native role discovery cannot be certified here.** I created project files, parsed locally, and researched documented discovery paths, but **did not run your real Codex installation**. `ADVISOR_DISCOVERY.md` requires actual native role registry/schema evidence and explicitly allows `NOT VERIFIED`. A configured TOML file alone is not runtime proof.
2. No universal, documented, non-spawning **list custom agent types** CLI command was verified. `codex agents`, `/agents` and `/subagents` display other things. Some Codex versions exposed delegation interfaces without a custom `agent_type` selector. See [reported regression #26363](https://github.com/openai/codex/issues/26363) and [recent fresh-session/daemon discovery issue #58063](https://github.com/openai/codex/issues/58063). These are **issue reports**, not proof that your installed client is broken; they justify a fail-closed discovery test and fresh session.
3. **Project root/trust:** While `.codex/config.toml` and `.codex/agents/` are documented project locations, the current live root/trust decisions cannot be assumed, particularly in a non-Git home configuration. Use the operator's effective configuration UI to establish load; do not silently insert global trust or root-marker settings.
4. **`project_doc_max_bytes=0` scope:** This only controls automatic project document inclusion. It does not make the Pi runtime's `AGENTS.md` unreadable, nor does it guarantee full filesystem access. That depends on workspace permissions and OS policy.
5. **Pi self-loading ancestor instructions:** Pi's documented context-file behavior means launching Pi with `cwd` under `~/.pi` may include the Codex-specific `~/.pi/AGENTS.md` as Pi context. Avoid such a working directory for Pi sessions if strict conceptual separation is desired. The Codex session's `-C` is separate from a Pi process's cwd.
6. **Task-specific testing boundaries are user rules**, not claimed as Codex or Pi universal requirements. No Bubblewrap, Pi SDK or interactive tests were run in this creation turn.
7. **Security and role capabilities:** The advisor's tools-disabled configuration and “analysis-only” instructions restrict behavior where supported but do not create an independent OS sandbox. Neither an agent role nor a Markdown rule can grant access past permission policy.
8. **No live Pi changes.** The files in this package are deployable proposals. Any `~/.pi/AGENTS.md` already present on the user's machine was not accessible here; deployment must back it up and reconcile user-specific prior instructions rather than overwrite blindly.

## 5. Deliverable-to-source mapping

| Artifact | Governing sources |
|---|---|
| `pi/pi-codex-reorganization-audit-token-efficient.md` | U1–U4, U8; P1–P5; C11 |
| `pi/AGENTS.md` | U5–U8; C1, C2, C5–C7, C11 |
| `pi/agent/.codex/config.toml` | U6–U7; C4–C6, C13 |
| `pi/agent/.codex/agents/pi_astra_advisor.toml` | U6–U7; C5–C7 |
| `pi/agent/.codex/advisor-workflow.md` | U6–U7; C1–C5, C8–C10 |
| `pi/agent/.codex/tests/ADVISOR_DISCOVERY.md` | U7; C3–C6, C8–C10, T1; reported issues noted above |
| `README-INSTALL.md` | U5–U7; C1–C5, C8–C10 and explicit live-environment unknowns |
