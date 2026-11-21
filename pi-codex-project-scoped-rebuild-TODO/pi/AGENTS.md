# Codex rules for maintaining Pi Coding Agent

**Scope:** These instructions are for Codex tasks involving `$HOME/.pi/agent/`. They are stored at `$HOME/.pi/AGENTS.md` and must be explicitly read when project `AGENTS.md` auto-loading is disabled. Codex's global instructions remain in `$HOME/.codex/AGENTS.md`. `$HOME/.pi/agent/AGENTS.md` belongs to **Pi Coding Agent**: treat it as configuration/documentation to inspect, not as Codex's own task instructions.

## Source of truth and working discipline

- Inspect the **live** Pi installation, its real package versions, source, configuration, loaders and writers. Old archives and Markdown plans are clues, not proof of current behavior. Do not invent APIs, paths, features, configuration keys, test results or implementation status.
- Prefer authoritative, version-matched installed source and documentation. Use Keenable and Context7 for research when helpful. Clearly distinguish observed behavior from assumptions and unknowns.
- Keep investigations targeted and token-efficient without sacrificing coverage. Implement evidence-based fixes, not speculative rewrites. Work in small, reversible batches with backups and explicit dependency/path updates.
- Respect Pi's documented file locations and extension discovery. Move a resource only when **all** of its readers, writers, reload paths and runtime dependencies will still work. Avoid accidental extension auto-loading and duplicate registrations.
- Preserve credentials, state, sessions, installed dependencies, permissions, sandbox restrictions and disabled-by-default MCP settings. No unapproved privilege expansion, secret exposure, destructive cleanup or third-party upgrade.
- Follow the active task's testing limits. Never claim live Pi functionality has been verified by static tests or by reading Markdown. Keep automated evidence and user-performed acceptance tests separate.

## Mandatory Sol-first / Astra-on-stuck advisory workflow

1. **Primary:** GPT-6.1 Sol (`gpt-6.1-sol`) owns analysis, implementation, testing and final decisions for Pi-maintenance work. The project-local Codex configuration selects it as the default. Do not change Codex's global model settings.
2. **Advisor:** The project-only native Codex agent `pi_astra_advisor`, defined at `$HOME/.pi/agent/.codex/agents/pi_astra_advisor.toml`, uses **GPT-6 Astra (`gpt-6-astra`)**. The advisor is an **integral required part of this workflow**, not an optional add-on, but **is consulted only when Sol is stuck**.
3. **Stuck means** Sol has made a focused, reasonable effort to understand or solve a technical problem, but cannot make meaningful progress or choose a defensible next step. Signs include repeated unsuccessful reasoning/fix attempts, unresolved architectural tradeoffs, or incompatible evidence despite targeted investigation. **Stuck does not mean** a tool is denied permission, the sandbox refuses an action, credentials are unavailable, a service is down, or user approval is required. Those are access/authorization/external conditions; the advisor cannot bypass them.
4. When stuck, Sol **must** send `pi_astra_advisor` a concise technical brief: goal, observed behavior, code/versions, approaches tried, why progress stopped, and precise questions. The advisor analyzes and recommends; it does **not** modify files, run commands, request privileges or take over the task. Sol reviews the advice, makes authorized changes and verifies them. A second consultation is justified only by materially new evidence.
5. If the project advisor is not discovered or cannot be used, report **`ADVISOR UNAVAILABLE`** and the actual reason. Do not silently substitute another model, invoke a generic Sol subagent under the Astra name, or pretend the consultation happened. Continue independent work when safe; if the stuck issue remains unresolved, report it as unresolved.
6. Project advisor setup and discovery must follow `$HOME/.pi/agent/.codex/advisor-workflow.md` and `$HOME/.pi/agent/.codex/tests/ADVISOR_DISCOVERY.md`. **Do not install any Pi-specific advisor or agent configuration under `$HOME/.codex/`.**

For the Pi reorganization task, obey the task prompt's separate limits on Codex-run tests, user-only runtime tests and final validation.
