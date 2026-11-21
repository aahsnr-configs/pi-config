# Operator tests: does Codex discover the project-specific Astra advisor?

**Goal:** establish whether Codex discovers `pi_astra_advisor` as an available *project-defined custom agent*. **This is not a test of model execution, model switching, advice quality or task outcomes.** Do not launch the advisor just to prove it is available. Run these tests yourself from a normal terminal/interactive Codex, not from a Codex-run Pi test.

Record for each step: `PASS`, `FAIL`, or `NOT VERIFIED`, the observed evidence, active Codex version, date and relevant trust/root information. The static Python check needs Python 3.11+ (`tomllib` is part of its standard library). A file existing or parsing is **not** sufficient evidence of runtime agent registration.

## A. Static installation checks (no Codex session or advisor invocation)

```bash
codex --version
python3 - <<'PY'
from pathlib import Path
import tomllib
home = Path.home()
root = home / '.pi' / 'agent'
project = root / '.codex'
config = tomllib.loads((project/'config.toml').read_text())
advisor = tomllib.loads((project/'agents'/'pi_astra_advisor.toml').read_text())
assert config.get('model') == 'gpt-6.1-sol', config.get('model')
assert config.get('agents', {}).get('enabled', True) is True
assert advisor['name'] == 'pi_astra_advisor'
assert advisor['model'] == 'gpt-6-astra'
assert advisor['description'].strip() and advisor['developer_instructions'].strip()
print('PASS: project files exist, TOML parses, intended identifiers/fields match')
print('NOTE: this does NOT establish native Codex agent discovery')
PY
```

If an installed global `~/.codex/agents/pi_astra_advisor.toml` exists, investigate a possible name collision. **Do not delete or edit it automatically.** Ensure the project definition is the intended one. A symlink pointing from the project to a global agent does not satisfy the required project-only ownership.

## B. Confirm Codex's actual workspace and project configuration

Launch a **new** session, not a resumed or stale-daemon thread:

```bash
codex -C "$HOME/.pi/agent" -m gpt-6.1-sol -c project_doc_max_bytes=0
```

Within Codex, use `/status` to confirm the current workspace/working directory is the expected `~/.pi/agent`. Use `/debug-config` if available in your installed CLI to verify that the **trusted project** `.codex/config.toml` is included in effective configuration layers. A CLI version may expose different commands; use `codex --help` and its installed documentation rather than assuming a missing command works. If the project is untrusted, authorize it through Codex's standard trust flow **only after reviewing the local project configuration**; do not alter global trust records by hand just to pass a test.

**Pass condition B:** the active project is correct and its project config is loaded. This confirms configuration scope, **not** registration of the advisor role.

## C. Inspect the registered agent roles without starting one

In the fresh session, issue:

> Read `$HOME/.pi/AGENTS.md` as my Codex-specific Pi maintenance rules. Do **not** create a subagent or run the advisor. Inspect the native delegation tools available to this session and report whether a **registered, selectable custom-agent type** named `pi_astra_advisor` is exposed. State the concrete tool/registry evidence. Do not infer registration from a readable TOML file, a task name, or the written instructions. If registration cannot be inspected, answer `NOT VERIFIED`.

If your Codex client exposes the native tool schema or available `agent_type` values, verify that `pi_astra_advisor` is among those values. The mere existence of a generic `spawn_agent` tool does not prove that the project role is selectable. Likewise `/agents` (chat command center), `/subagents` (active threads), `/status` and `/debug-config` are **not** documented as custom-role registries. Don't count any of them as proof unless the interface itself explicitly shows registered custom-agent types.

**Pass condition C:** an actual native role/selector registry or tool schema exposes `pi_astra_advisor`, with clear evidence. If no non-spawning discovery interface is available, mark `NOT VERIFIED` rather than `PASS`; that is a product inspection limit, not necessarily a broken install. If schema shows no custom agent selector, mark `FAIL` for reliable selection on this installed client.

## D. Negative checks and corrective actions

- Confirm the project-only definition is **not being introduced** under global `$HOME/.codex/agents/`.
- Confirm `project_doc_max_bytes=0` has **not** been mistaken for filesystem hiding. From a permitted shell in Codex, a read-only listing of `$HOME/.pi/agent/` should still work when sandbox permissions permit it. Do not read or print `auth.json` contents.
- If the custom agent cannot be discovered, inspect installed Codex version, project root, project trust, TOML parse errors, possible role-name collisions, and stale sessions/daemon state. Start a fresh session after corrections. Do **not** switch the primary to Astra and do **not** pretend naming a generic subagent `pi_astra_advisor` uses the custom role.
- Do not use a real Astra invocation as a substitute for these **discovery-only** tests. If the native client provides no non-spawning registration inspection, clearly report the limit and ask the user whether they separately want a bounded activation probe; that would be a **different test**.

## E. Confirm Sol understood the mandatory advisor rule without invoking it

In the same session, ask:

> Using my `$HOME/.pi/AGENTS.md`, explain precisely **when** you are required to consult the project `pi_astra_advisor` role, and why a permission denial, sandbox refusal or missing credential is **not** the same as being stuck. Do not spawn an agent and do not change models.

**Pass condition E:** Sol states that it performs ordinary work itself, is required to consult Astra when actually **stuck in the task**, distinguishes access restrictions from inability to progress in reasoning, and refuses to use a generic fallback in place of the named custom role. This verifies understanding of the instruction, **not** runtime advisor detection.

## Results

| Check | Status | Evidence |
|---|---|---|
| A. Local project TOML correct | NOT RUN | |
| B. Project config and trust active | NOT RUN | |
| C. Custom advisor role visible in native registry | NOT RUN | |
| D. Scope, collision and file-visibility checks | NOT RUN | |
| E. Sol understands stuck-versus-access distinction | NOT RUN | |

**Acceptance:** A+B+D+E pass and C is supported by direct native discovery evidence. Otherwise the claim “Codex detects Astra as advisor” remains unverified or failed; do not overstate it.
