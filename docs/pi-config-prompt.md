# Pi Coding Agent — Comprehensive Configuration Audit, Reconstruction, and Repackaging

## 1. Role, input, and outcome

Act as a senior Pi Coding Agent architect, TypeScript/JavaScript engineer, security auditor, and systems-integration specialist. **Input:** the attached `pi-config.zip`, intended to contain my complete, sanitized Pi configuration, including `pi-extensions.txt` (installed-extension list). Its apparent root is `pi-tmp/agent/`, which may be a staging layout rather than an installable path. **Do not assume** that root or any file placement is correct.

**Mission:** examine **every** archive entry and its relationships; establish each component's individual and subsystem-level purpose; research applicable Pi and extension behavior; correct misplaced, obsolete, duplicated, divergent, incomplete, misconfigured, insecure, or broken components; repair all affected dependencies; and verify that the final configuration preserves intended behavior, permissions, security boundaries, performance, and maintainability.

**Mandatory primary deliverable:** a **modified, fully reconstructed version of the attached ZIP**, named `pi-config-reconstructed.zip`, containing the final corrected Pi configuration (not merely a suggested layout or reports). Keep `pi-config.zip` unchanged. Include all required custom files and supporting resources, correctly organized for the documented installation method. The archive must be internally consistent, complete for its intended scope, independently extractable, and validated **after packaging**. Supply a download link/path to this ZIP in the final response. Reports and change manifests are additional deliverables, not substitutes for it.

Make only evidence-backed changes necessary for correctness, integration, security, cleanup, and measured efficiency; do not undertake unrelated redesigns or feature additions. Treat this as a software-system audit, **not** cosmetic file cleanup.

## 2. Scope and inventory

Inventory all nested files, directories, hidden items, symlinks, binaries, documentation, test fixtures, backups, generated artifacts, disabled components, and state. Known examples (not exhaustive):

- Core/config: `AGENTS.md`, `settings.json`, `mcp.json`, `mcp-on-demand.json`, `sandbox.json`, `models.json`, `models-store.json`, `keybindings.json`, `fast-tui.json`, `promptsmith-settings.json`, `pi-extensions.txt`.
- Agents/workflows: `agents/`, `pi-subagents/`, `extensions/subagent/`, `workflow/`.
- Extensions/related subsystems: `extensions/`, `extensions/pi-mcp-on-demand/`, `pi-mcp-on-demand/`, `extensions/opencode-ui-evidence/`, `permission-mode/`, `prompt-review/`, `skills/`.

For each entry record: exact original path; type; owner (Pi core, third party, custom, support); intended purpose; callers/consumers; expected canonical path and discovery method; active/conditional/disabled/development/unused status; integrity; exact/semantic/intentional-duplicate status; security implications; action and evidence. Capture the source directory tree and a machine-readable inventory. Identify subsystem responsibilities, triggers, inputs/outputs, side effects, settings, dependencies, and shared state.

Before changes, establish the available **environment baseline**: Pi version, OS/terminal, runtimes/package managers, package/extension versions, module-loading rules, global/project paths, configuration precedence/overrides, relevant environment variables, and dependencies. If the live installation is inaccessible, label archive-only conclusions as static findings and do not claim runtime verification.

## 3. Research and evidence policy

Use **Keenable** for focused web/upstream research and **Context7** for current library/framework documentation when needed. Verify the Pi version's official docs/source; extension APIs, manifests, loading paths, lifecycle, configuration formats, changelogs and breaking changes; versions of third-party extensions in `pi-extensions.txt`; external libraries and runtime/platform behavior; relevant maintainer tests/issues. Where Context7 lacks a package, consult its authoritative upstream repository/docs. If either tool is unavailable, disclose the limitation and use authoritative alternatives. **Never claim to have used a tool you did not access.**

Evidence order: version-matched official documentation/source; installed implementation and observed behavior; upstream tests, release notes, and maintainer guidance; credible technical references; community observations only as supplementary evidence. For every uncertain technical decision, record the question, version/commit where applicable, sources, findings, and confidence. Separate **verified facts**, **source-code observations**, **inference**, and **unresolved uncertainty**. Never invent paths, APIs, options, command syntax, version behavior, or tests. Preserve uncertain files and report blockers rather than fabricate fixes.

## 4. Nonnegotiable operating controls

- **Safe intake:** inspect ZIP integrity, extraction bounds, zip-slip paths, suspicious executables, and unsafe symlinks; extract into isolated staging; never run archive code before reviewing it.
- **Preservation:** protect existing custom behavior, intentionally disabled modules (`.disabled` included), session/state files, tests, documentation, backups, and recoverable artifacts when still needed. Do not assume unreferenced files are unused: inspect dynamic loading and conditional paths.
- **Reversibility:** record original locations/content, consumers, reasons and rollback before changes; use backups/version-control checkpoints and quarantine for potentially obsolete items. Avoid blanket cleanup commands; never silently overwrite, delete irreplaceable data, or change credentials. Obtain approval for irreversible/potentially valuable deletion, live destructive operations, security relaxations, or material intent ambiguities. Routine, reversible, well-evidenced edits need no repeated approvals.
- **Security:** preserve or strengthen sandboxing, per-agent and child-agent tool/file/network permissions, MCP consent/activation policy, and authorization boundaries. Do not broaden host privileges or bypass isolation to pass tests. UI restrictions alone are not enforcement. Avoid unsafe shell interpolation or uncontrolled access. No secret values in outputs/logs/ZIP; inspect for accidental sensitive data despite the archive being described as sanitized.
- **Change scope:** avoid silent behavior changes, dependency upgrades, custom rewrites, or functional simplification. If necessary, explain the compatibility impact, reason, validation, and rollback. Preserve separate source/deployment/entry-point copies **when required**.
- **Efficiency:** analyze in dependency order; avoid repeated full-file dumps, redundant searches, excessive log/context injection and test reruns, but never trade away correctness, feature coverage, or security for speed/token savings.

## 5. Required tasks

### Task 1 — Locate every component correctly

Verify canonical global and project-local Pi directories, extension source/entry points, extension-owned config/state, agents, skills, workflow modules, tests, development data, documentation and caches. Determine whether paths are hardcoded, relative, environment-dependent, package-resolved, or runtime-discovered; check manifest registration, symlinks, loader behavior and precedence. Distinguish core config, third-party config, custom implementation, project-specific configuration, generated state and historical artifacts. Produce a proposed canonical tree and **old-to-new path map** with evidence. Relocate only after auditing dependencies; update all affected imports, file references, commands, manifests, test fixtures and documentation. Do not assume `pi-tmp/agent/` should be installed verbatim under `~/.pi/agent/`.

### Task 2 — Safely remove what is truly unnecessary

Classify entries as **required**, **conditionally required**, **optional but valuable**, **generated/reproducible**, **proven redundant**, **obsolete**, **unused after investigation**, or **unknown**. Inspect backup snapshots, caches, screenshots/UI evidence, generated files, disabled modules, legacy examples, stale reports, orphaned code and test fixtures; preserve anything needed for maintenance, rollback, regression tests or reproducibility. Examine `extensions/opencode-ui-evidence/` particularly carefully: screenshots, backups and historical materials are **not automatically disposable**. Likewise, do not discard `models-store.json`, state, or cached metadata solely because it looks generated. For each removal candidate, identify direct/indirect/dynamic consumers, prove replacement or irrelevance, preserve a recoverable copy, remove it from active paths only when safe, and rerun affected checks. **Never delete unknown files merely to reduce clutter.** Record retain/quarantine/remove decisions and restore procedures.

### Task 3 — Detect duplicates and reconstruct conflicting files

Use hashes plus semantic comparison to distinguish (a) identical copies, (b) same-name/different-content conflicts, (c) functionally overlapping modules, (d) intentional source/deployment/fixture replicas, and (e) partially divergent implementations. Prioritize the relationship between `extensions/pi-mcp-on-demand/` and `pi-mcp-on-demand/`: inspect entry points, registries, policies, prompt matching, configuration, loader paths and versions to determine whether they are deployment/source copies, competing versions, migrations, or deliberately independent modules. Apply this rigor to **all** other candidates.

For divergent copies, review content, history/docs, callers and upstream behavior; inventory unique features, fixes, edge-case handling and security checks; choose the canonical implementation **based on evidence**, not file size or apparent recency. Integrate compatible intended behavior without blind concatenation. Preserve separate required entry points backed by shared code if appropriate. Remove redundant active copies **only after** redirects, tests and duplicate-hook/tool/command-registration checks pass. Produce a duplicate/conflict matrix showing decisions, preserved functionality, alternatives and evidence.

### Task 4 — Repair dependencies and end-to-end integration

Repair errors introduced by restructuring **and pre-existing defects**. Check schemas, keys/types/defaults, syntax, configuration precedence, required fields, missing libraries/modules, imports/exports, paths, lifecycle initialization/teardown, resource cleanup, command/keybinding/tool/hook registrations, event ordering, duplicate execution, errors and compatibility with installed versions. Do not infer that a valid JSON file is a valid Pi/extension configuration.

Audit these cross-component contracts:

1. **Global configuration:** `settings.json`, `AGENTS.md`, models and state files, MCP and sandbox configs, agent/skill/workflow definitions, permissions, UI/keybindings and other discovered files; verify correct ownership, discovery, layering and overrides.
2. **Custom extensions:** Pi API support; module resolution; initialization/shutdown; registrations and collisions; conditional/disabled loading; dependency and failure handling. Prevent double loading.
3. **Agents/orchestration:** `agents/`, `pi-subagents/`, `extensions/subagent/`, `workflow/`; identify intended delegation architecture, manual/automatic activation, role/tool permission scopes, handoffs, state/context preservation, task dependencies, completion verification, recovery, races, duplicate work, child process cleanup, and privilege inheritance. Do not replace a custom orchestrator with generic subagents without proving equivalence.
4. **MCP on demand:** `mcp.json`, `mcp-on-demand.json` and related source; validate disabled-by-default intent, deterministic prompt matching, user consent, temporary request/session activation, safe deactivation, isolation, concurrency, failures/shutdown, duplicate implementations and **no silent persistent modification/enablement of `mcp.json`** when that is the intended policy.
5. **Permissions and sandbox:** check overlapping enforcement layers and precedence; ensure child agents cannot escalate or bypass restrictions through alternate tool routes; preserve network/filesystem/shell boundaries, session scoping and any separately authorized emergency host-access override.
6. **Skills/prompts/UI/model tools:** validate metadata, discovery, references, commands, runtime output, prompt guards, recursion risks, terminal capability assumptions and appropriate tool visibility. Preserve custom UX features and intended functionality.
7. **Efficiency:** assess startup latency, duplicated initialization/hooks/MCP schemas, eager loading, large static prompt injection, excessive subprocesses, background tasks, filesystem rescans and context use. Apply only conservative **measurable** optimizations.

Map subsystem dependencies and cross-effects; distinguish legitimate overlapping security layers from accidental duplication. Document all repaired references, tested integrations and remaining conflicts.

### Task 5 — Explain architecture, purpose and correctness

For each component and logical group document: purpose/rationale; triggers; inputs/outputs/side effects; owners; configuration; callers/callees; optionality; implementation-versus-intent; documentation and test coverage. Include subsystem maps for orchestration, permissions/sandboxing, on-demand MCP, prompt/skill management, UI/terminal integration, model management, performance, and tests/evidence. Check shared state, command collisions, ordering, competing policies, missing integrations, and cleanup/teardown.

Identify missing files, imports, declarations, fixtures, scripts, configs and dependencies; distinguish genuinely missing custom assets from external packages, optional resources, runtime-generated content and stale references. Rebuild custom missing content **only if behavior is sufficiently evidenced**. Audit syntax, API usage, TypeScript/JavaScript logic, error cases, dead code, path resolution, shell safety, state transitions, races, cleanup, partial failure recovery, permissions and documentation accuracy. When code and docs disagree, investigate authority rather than arbitrarily selecting one. Record irreducible uncertainty honestly.

## 6. Ordered execution and approvals

1. **Discover:** validate/extract safely; inventory and hash everything; establish version/environment baseline; read implementations, configs, `pi-extensions.txt`, tests and docs.
2. **Research:** validate version-specific paths/APIs/behavior with Keenable, Context7 and authoritative sources; create dependency/subsystem graphs.
3. **Diagnose:** propose canonical layout; classify unnecessary/unknown entries and duplicates; trace all consumers; identify bugs, security gaps, performance waste and missing dependencies.
4. **Plan:** create an actionable change manifest, old/new path map, duplicate merge decisions, impacted tests, rollback/checkpoints and any approval-required decisions.
5. **Implement:** make small, reversible, dependency-ordered edits in staging; inspect outputs, update every consumer, run targeted checks after each substantial change. Do not alter the live configuration unless explicitly authorized and necessary.
6. **Validate:** run static checks, dependency resolution, safe integration/security tests, regression and runtime checks where an environment exists; repair failures and rerun affected checks.
7. **Package:** produce and independently validate the **modified ZIP** as specified in Section 8; deliver it with audit records and clear installation/rollback instructions.

Do not interrupt repeatedly for routine reversible decisions. If a material choice is blocked, preserve the affected component, document options and evidence, and continue with independent tasks. Never mark a blocked or unrun check as passed.

## 7. Verification requirements

Use **actual project-supported** commands, never invented test commands. Report environment, method/command, expectation, observation/evidence and **pass/fail/blocked/not-run** for each check.

- **Static:** validate applicable JSON and extension schemas, TS/JS/Python/shell syntax and metadata, supported API/options, module exports and referenced paths; check unreachable/dead code, invalid assumptions and shell safety.
- **Dependencies/registration:** ensure every required module/resource resolves; extension entry points and manifests are correct; commands/tools/keybindings/hooks are registered exactly as intended, not twice; relocated files are discoverable.
- **Integration/regression:** verify Pi startup and extension initialization (if available), agent delegation/workflow completion/handoffs/recovery, permissions and sandbox, temporary MCP lifecycle, prompt/skills, UI/terminal behavior, configuration persistence, failure paths and state cleanup. Compare with original intended behaviors, not only the modified system.
- **Security:** in harmless isolated fixtures, test rejected unauthorized access/tool use, child-agent restrictions, sandbox boundaries, authorization/override, no MCP persistent enablement and bypass resistance. Never conduct destructive tests on live data.
- **Performance:** when proposing optimizations, measure relevant startup/work/token overhead before and after, or report that measurement was unavailable.
- **Integrity:** detect unexpected data loss, disabled-feature activation, behavior regressions, duplicate registrations and accidental credential disclosure.

Clearly distinguish **static review**, **executed test**, and **unverified runtime assertion**. Preserve evidence; resolve all feasible failures. No global “error-free” or “production-ready” claim without adequate proof.

## 8. Final ZIP — required primary artifact

Create `pi-config-reconstructed.zip` **from the finalized staging tree**, not the unchanged input. Requirements:

1. **Complete and installable:** include every necessary corrected configuration, custom module, agent, skill, script, test/resource and supporting file within the intended distributable scope. Preserve intentional hidden files, permissions, symlinks (only when safe), executable bits and disabled states. Do not accidentally exclude modules loaded dynamically.
2. **Canonical layout:** package a consistent archive-relative directory tree; remove obsolete active duplicates; document exactly which archive directory maps to `~/.pi/agent/` or other verified destination(s). Do **not** assume archive root equals install root. If a distinct source tree or project-level path is legitimately required, include and explain it.
3. **Documentation:** bundle the audit/reconstruction/research/validation/rollback reports and inventory/manifest under a clearly non-runtime documentation location such as `audit/` at the archive root (or another verified safe location). Do not place reports where Pi would interpret them as skills, extensions, or agent instructions. Include a `README-INSTALL.md` with prerequisites, version compatibility, placement/merge directions, backup, smoke tests, and rollback. Avoid embedding raw logs or other unnecessary bloat.
4. **Security and hygiene:** exclude secrets, credentials, absolute private paths where avoidable, unintended session/user data, unsafe symlinks, temporary extraction debris, quarantine-only files and unnecessary reproducible caches. Do **not** silently drop a file whose data is needed for correct runtime behavior. Keep full backups/quarantine separate from the distributable ZIP unless a specific recovery asset is deliberately required and safe to ship; document exclusions.
5. **Manifest and traceability:** list original-to-final paths, retained/changed/merged/removed files, content hashes, dependencies, evidence and known limitations; distinguish intentional omissions from losses. Include archive checksum and summary outside the archive as appropriate.
6. **Post-package verification:** re-open and re-extract the new ZIP into a **fresh** temporary directory; check archive integrity, zip-slip/symlinks, required paths, complete manifests, hashes, referenced imports/config paths and absence of accidental duplicates/secrets. Execute feasible static/smoke tests **against the extracted final package**, not only the pre-archive staging folder. Confirm it differs from the input as documented. Any packaging-induced failure must be repaired and the ZIP rebuilt.
7. **Deliver it:** link the exact verified `pi-config-reconstructed.zip` in the final answer; do not substitute individual files, an unmodified copy of the original ZIP, or a plan to generate it later. If blockers prevent a safe complete build, explain them precisely and label any partial ZIP as **partial/unvalidated**, not complete.

## 9. Required reports (also package in the ZIP)

| Artifact | Required contents |
|---|---|
| `PI_CONFIG_AUDIT.md` | Initial architecture and structural/config/security defects, misplaced/unused/missing components and findings. |
| `PI_CONFIG_INVENTORY.json` | Every archive entry: type, owner, purpose, original/final locations, consumers, duplicate class, status, action, evidence. |
| `PI_CONFIG_ARCHITECTURE.md` | Component/subsystem catalog, responsibilities, interactions, architecture and dependency diagrams (Mermaid if useful). |
| `PI_CONFIG_RECONSTRUCTION.md` | Original vs. final trees, path mapping, merges, rebuilt files, reference/dependency fixes and rationale. |
| `PI_CONFIG_CLEANUP.md` | Retained, moved, merged, quarantined and removed items, proof of safety, preservation/rollback. |
| `PI_CONFIG_VALIDATION.md` | Test matrix, executed commands, expected/actual behavior, passes/failures/blocked/not-run, evidence and limitations. |
| `PI_CONFIG_RESEARCH.md` | Technical questions, authoritative sources, version/commit matching, Keenable/Context7 findings and uncertainties. |
| `PI_CONFIG_ROLLBACK.md` | Exact restoration/undo steps and preserved backup/checkpoint locations. |
| `PI_CONFIG_FINAL_REPORT.md` | Initial/final assessment, changes, remaining issues, package contents/installation and issue-resolution matrix. |
| `README-INSTALL.md` | Verified archive layout, installation placement, prerequisites, safe backup/merge, smoke test and rollback instructions. |

Final report's issue matrix must include **ID, component, issue, root cause, corrective action, validation, final status**. Include known blockers and future maintenance recommendations. Keep reports out of Pi runtime-discovered directories.

## 10. Acceptance criteria (evidence required)

- [ ] Every original entry inventoried and classified; each component's purpose established or uncertainty noted.
- [ ] Major subsystems, dependencies, behavior and intended activation paths documented.
- [ ] Pi version and relevant upstream conventions researched; canonical paths, config precedence and APIs verified.
- [ ] Misplaced entries corrected; old-to-new paths and all dependent references reconciled.
- [ ] Redundant/obsolete files safely assessed; unknown/valuable data retained; removals recoverable.
- [ ] Exact, semantic, intentional and conflicting duplicates resolved appropriately; unique features/edge cases retained; no unintended double registration.
- [ ] Missing dependencies, invalid options/schemas, faulty imports/exports and identified defects corrected or explicitly blocked.
- [ ] Agents, orchestration, MCP-on-demand, permissions/sandbox, skills/prompts, tools, models and UI interactions checked without security or functionality regression.
- [ ] Static, integration, regression, security and runtime tests performed where possible; other checks accurately marked blocked/not run.
- [ ] All significant changes, research evidence, exceptions and rollback steps documented.
- [ ] **`pi-config-reconstructed.zip` created, freshly re-extracted, security/integrity-checked and validated as the actual deliverable; includes the complete corrected configuration and separate audit/install documentation.**
- [ ] Final answer supplies a verified download link/path for the modified ZIP, plus an honest summary of unresolved issues.

## 11. Final response format

Provide, in order: **(1)** overall before/after assessment; **(2)** key directory changes and canonical install mapping; **(3)** duplicate/merge decisions and preserved functionality; **(4)** cleanup summary; **(5)** dependency/security/integration repairs; **(6)** executed validation with status and evidence; **(7)** remaining issues/blockers; **(8)** the verified **modified ZIP download link**, its checksum and installation instructions, plus supplementary report locations; **(9)** consolidated issue/root-cause/fix/verification/status table.

**Success means a corrected, coherent, recoverable, validated configuration delivered inside the modified ZIP—not just analysis. Understand first, verify second, change minimally, test continuously, and check the packaged artifact itself.**
