# Project-only Codex advisor workflow: Sol 6.1 → Astra 6 when stuck

## Contract

- **Primary model:** `gpt-6.1-sol` for all normal Pi-maintenance work.
- **Advisor agent:** `pi_astra_advisor` using `gpt-6-astra`, defined in this project's `.codex/agents/pi_astra_advisor.toml`.
- **Location:** all advisor configurations live under `$HOME/.pi/agent/.codex/`, not under global `$HOME/.codex/`.
- **Status:** the advisor is a required **available** capability of this project. Its **use is conditional on Sol becoming stuck**, not a mandatory extra step for each task. The parent must never quietly replace it with a generic agent.

## Distinguish stuck from external obstacles

**Stuck (consult Astra):** Sol has inspected relevant code/evidence and attempted a reasonable line of reasoning or repair yet cannot progress; repeated attempts converge on the same failure without understanding; two plausible technical approaches have unresolved tradeoffs; or conflicting evidence prevents a safe decision.

**Not stuck (do not consult Astra merely because of this):** a permission prompt was denied; sandbox/file access is restricted; Pi must be tested by the user; credentials or model access are unavailable; network/server is down; a command needs authorization; a required file is missing. Report these conditions or request the appropriate authorized action. An external obstacle can coexist with a separate reasoning problem; consult Astra **only** for that reasoning problem.

## Procedure

1. Sol investigates relevant implementation paths, checks evidence and makes a bounded attempt to resolve the technical uncertainty. Do not repeat identical unsuccessful approaches.
2. If still **stuck**, Sol prepares a brief of only the necessary material: expected behavior, observed failure, exact relevant code and versions, what was tried, why progress stopped, key constraints, and focused questions. Exclude credentials and unrelated private data.
3. Sol delegates that brief to the **registered custom agent type** `pi_astra_advisor`. If the available delegation tool cannot select this type, **do not** spawn a generic agent or use a similarly named task as a substitute. State `ADVISOR UNAVAILABLE`, include the missing capability, and leave that question unresolved until corrected.
4. The Astra advisor provides recommendations only. Sol evaluates them against installed code, makes authorized changes, performs only allowed tests, and records whether the problem was solved. A further consultation requires genuinely new evidence.
5. If escalation fails, Sol continues unrelated safe work but does not manufacture a solution or claim the advisor was consulted.

The TOML's tool-limiting feature keys provide defense-in-depth where supported; **they are not proof of a separate sandbox**. Custom agents inherit parent permissions, and the task's prohibitions remain binding for both agents.

## Instruction discovery versus file access

This workflow assumes the task explicitly told Codex to read `$HOME/.pi/AGENTS.md`. Launching Codex with `-c project_doc_max_bytes=0` prevents automatic project `AGENTS.md` injection, including Pi's `agent/AGENTS.md`, but **does not hide files** and does **not** disable project `.codex/config.toml` or custom-agent discovery. Project-local configuration still depends on project trust, the effective Codex version, and the discovered project root. Verify those conditions before claiming the advisor is available.

The paired `tests/ADVISOR_DISCOVERY.md` checks **discovery**, not whether Astra produces better advice and not the model of a spawned agent. No advisor invocation is necessary for baseline checks. If the installed Codex build does not expose a reliable way to inspect registered custom-agent names without spawning, report that as **not conclusively verified**; do not fabricate a pass.
