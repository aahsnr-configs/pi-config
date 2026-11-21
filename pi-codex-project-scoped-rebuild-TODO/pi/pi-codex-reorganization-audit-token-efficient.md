# Codex implementation task: declutter, audit and repair my Pi Coding Agent configuration

**Execution target:** the **existing live** `$HOME/.pi/agent/`, not a clean installation or the old reconstructed ZIP. **First read and follow** `$HOME/.pi/AGENTS.md` as my Codex-specific instructions. The file `$HOME/.pi/agent/AGENTS.md` is **Pi Coding Agent's own instructions** and is part of the configuration to inspect; do not adopt it as Codex operating policy. Respect the project-specific Sol/Astra-on-stuck workflow defined in the Codex AGENTS file. Do not relocate or globalize the project-local `.codex/` advisor files.

## Goal

Implement the most complete **safe and compatible** reorganization of my Pi configuration; then comprehensively audit, identify and repair errors, omissions and incorrect integrations. The finished Pi configuration must work as one system with its installed extensions. Do not merely write a plan or trust existing Markdown as proof that features exist.

### 1. Reorganize JSON files

- Keep `$HOME/.pi/agent/auth.json` and `$HOME/.pi/agent/settings.json` at the root.
- Examine **every other JSON file**, regardless of whether Pi, a third-party extension, an installed package, a custom module, or another process created it. For **each**, identify its owner; purpose; current path; exact readers/writers; discovery, save, refresh and reload behavior; runtime sensitivity; and dependencies.
- Move **every JSON file that can genuinely be relocated** into an appropriately named folder under `$HOME/.pi/agent/` (prefer `config/`, organized further only when useful). Update **all** consumers and persistence paths. Do not assume an arbitrary JSON file can be redirected by `settings.json`.
- For any file that Pi core or a third-party extension requires at a particular location, **retain the necessary path** and document the reason. Verify a supported redirect before moving it. Do not invent settings keys, break atomic writes with unverified symlinks, modify vendor source solely for aesthetics, or sacrifice functionality to force an empty root.

### 2. Organize custom code

- Locate **all custom code** across the agent tree, not merely files already in `extensions/`. Reorganize it into suitably named directories under `$HOME/.pi/agent/extensions/` wherever compatibility permits.
- Separate active extension entry points from supporting libraries, workflow launchers, scripts, utilities, tests and inactive development sources. **A file's physical location must not accidentally activate it** as a Pi extension; verify the installed Pi loader's actual discovery rules and configured entry points.
- Update all imports, module paths, shebangs, commands, runtime working-directory assumptions, launchers, package manifests, tests, documentation and settings entries affected by moves. Keep original functionality and initialization order; do not create duplicated registrations or conflict between source and deployed copies.
- For code that cannot safely move there without redesigning how Pi discovers/executes it, explain the verified exception and preserve a working entry point.

### 3. Make every integration work after relocation

- Inspect actual Pi version, `settings.json`, installed packages, `pi-extensions.txt`, package manifests, startup discovery, project overrides, configuration/state paths and all extension registrations. Use `pi-extensions.txt` as an inventory to **reconcile**, not as proof that every listed package currently loads.
- Ensure **Pi itself and every installed extension** can still locate its required configuration, executable, package, code, resources and persisted state. Update `settings.json`, the Pi-specific `$HOME/.pi/agent/AGENTS.md`, and every affected file only through mechanisms demonstrably understood by the actual code.
- Check interactions across all discovered components: permission modes, sandbox, child agents, custom agents/workflows, MCP registration/on-demand activation, prompts, skills, prompt review, model/provider configuration, terminal UI, sessions and other installed functionality. Investigate duplicate or apparently competing implementations before deciding what may be consolidated.
- Preserve **security behavior and user decisions**: least privilege, sandbox boundaries, denied paths, MCP servers disabled by default unless explicitly authorized, session state, credential protection and manual approvals. Do not broaden access or silently enable a capability to make validation pass.

### 4. Audit the entire Pi installation, not just documents

- Create a complete, reproducible inventory of the live tree, including hidden files, symlinks, JSON, code, installed packages and **every Markdown file recursively**. Read/analyze each Markdown file (efficiently index/search large trees), then check its commands, paths, promises, dependencies and described behavior against the **real implementation**.
- Independently inspect **all code and configuration** for correctness and completeness with respect to the **whole assembled Pi system**, including features not mentioned in Markdown. Identify broken or missing implementations, invalid references/schema, stale paths, unsafe defaults, duplicate registration, permission conflicts, missing dependencies, startup incompatibilities and untested behavior.
- Treat comments, task plans, tests and archived reports as hypotheses until the relevant implementation and allowed checks substantiate them. Fix demonstrated defects within scope; describe anything not safely provable, reproducible or repairable. No invented bug fixes or false claims of completeness.
- Prefer installed source and version-matched docs; research with Keenable and Context7 when needed. Record important findings and their supporting code/docs in the final audit report, not scattered citation text in this prompt.

## Safety and implementation method

1. **Inventory and trace.** Before edits, catalog JSON/code/Markdown, owners, readers, writers, active and inactive entry points, exact file movements, path dependencies, and cases that must remain in place. Check installed versions and available test tools; inspect code rather than guessing.
2. **Back up and plan reversibly.** Preserve content, permissions, useful state and secrets. Create a timestamped recoverable backup before moving/editing anything. Work in small dependency-ordered groups; no uncontrolled deletion, unapproved third-party package upgrade or side effects outside the relevant Pi paths. Do not print, upload or place secrets in audit artifacts.
3. **Implement and review each group.** Move only supported paths; update every reader, writer and reference; perform allowed local checks; scan for stale references and regressions; keep manual rollback steps.
4. **Full-system independent audit.** Check not only whether documents describe implementation, but whether existing implementation fulfills the user's intended system-wide functionality and security. Repair verified problems and document exceptions.
5. **Separate test tracks and final validation.** See below. Do not claim the whole configuration is fully working until the operator performs live acceptance tests.

Optimize Codex token use via inventory/hashes, targeted searches, compact evidence and incremental verification. Never use token efficiency as a reason to skip files or necessary analysis.

## Tests: strictly separated tracks

**Codex-run checks (allowed):** Safe, noninteractive static analysis; JSON/TOML/YAML parsing; import/path and reference resolution; schema validation using installed schemas; package/manifest consistency; targeted typechecking/lint where safe; offline pure-function unit tests that do **not** launch Pi, child agents, live providers/MCP or interactive applications. Review each command and its transitive test runners first. Use mocked fixtures as appropriate, labeling mock evidence accurately.

**Codex-run prohibitions:** Do **not** run the Pi CLI application or SDK runtime in a sandbox, launch live Pi sessions, Pi child/workflow/orchestrator processes, Kitty UI, real MCP servers/provider calls, interactive tests, `bwrap`/Bubblewrap or scripts that transitively invoke any of them. Do not alter/disable sandbox or permissions to make a test pass. If a check would require this, document it for the user rather than executing it.

**User-run acceptance tests:** Create `$HOME/.config/pi/Tests/` and **separate thorough Markdown procedures** for every relevant area and extension found in the live installation. Include an index/README and common setup, known prerequisites, exact commands/interactions, expected results, negative/security cases, exit criteria, result-recording forms and cleanup/recovery. Cover launching **normal Pi with the complete installed extension set**, startup/loading/absence of collisions, configuration after relocation, model/auth behavior without revealing secrets, all active extensions, custom code/commands, agents and permissions, sandbox boundaries, MCP consent/revocation, workflow/orchestrator, UI/Kitty, sessions/reload/resume, failure recovery and all remaining interactive/integration/security checks Codex cannot perform. Do not create an arbitrary number of test files; determine coverage from the actual inventory. **Do not run these tests for the user.** Mark them `NOT RUN`.

**Post-user-test defects:** Results and defects from `$HOME/.config/pi/Tests/` constitute a **separate future remediation track**. Do not invent their outcomes or silently preempt/fold hypothetical live failures into Codex's current automated validation.

**Final Codex validation (distinct from user tests):** After completing permitted changes and audits, independently rerun safe parsers/schema checks, import/path/manifest consistency, static permission-policy invariants, stale-path scans, duplicated-registration checks, Markdown link/path/command checks, change/rollback reconciliation, and any appropriate pure offline tests. Report `PASS`/`FAIL`/`NOT RUN` and evidence. If a validation check cannot run, keep it unresolved; do not replace it with invented results or a live Pi test.

## Deliverables and completion criteria

- Working reorganized **live** `$HOME/.pi/agent/` with all safely movable JSON and custom code relocated and all affected consumers corrected; documented fixed-path exceptions and unmodified assets.
- A full move map and dependency/consumer inventory: original → new path, owner, reason, readers/writers, action, exceptions and remaining risks.
- Complete audit findings and repairs (including Markdown-to-implementation mismatches, implementation gaps not covered by Markdown, compatibility/security review, and unresolved issues).
- Backup location, exact per-file change manifest, non-destructive rollback procedure and paths preserved for Pi/core/installed extensions.
- Separate automated-check evidence, **independent final validation** report, and user-only acceptance suite under **`$HOME/.config/pi/Tests/`** (README plus component-specific Markdown cases), all user-only results pending.
- A concise handoff: what was changed, what was demonstrably checked, what remains unknown, and the exact tests I must personally run before declaring production readiness.

**Done** means every proposed movement/fix is justified by observed source, valid paths and allowed evidence; features/security are preserved as far as the allowed checks can establish; each unverified runtime behavior has a manual test; and no untested claim is presented as a pass. If a required move would break Pi or an extension, keep its working location and report why.
