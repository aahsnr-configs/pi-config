# Pi Agentic Workflow Setup: Subagent Extension Analysis

*Research snapshot: information current to roughly Nov 14, 2026 (one listing below is dated Nov 15).*

**Goal:** a 4-role workflow (Planner/Orchestrator, Executor/Worker, Reviewer, Verifier) for the pi coding agent, plus separate `/plan` and `/build` modes and other custom agents.

**Sourcing note:** GitHub blocked automated access, so this draws on pi.dev package pages (which mirror the READMEs), npm snippets, and community write-ups. gotgenes' own "comparison with upstream" doc could not be retrieved. Anything marked *verify* is untested.

---

## 1. Bottom line

- **Best subagent engine for this workflow:** `nicobailon/pi-subagents` (npm: `pi-subagents`). It has the workflow machinery built in: chains, parallel runs, review loops, worktrees, fleet view.
- **Best minimal, typed foundation:** `@gotgenes/pi-subagents`. It is cleaner and smaller, but you would write the orchestration layer yourself.
- **Neither provides `/plan` and `/build`.** Add a separate extension (section 4).
- **Recommended stack:** `pi-subagents` (nicobailon) for subagents, plus `pi-agent-mode` for primary-agent switching (or `@janvitos/pi-plan-build` if you want a hardened plan/approve/build flow).

---

## 2. The two candidates

### @gotgenes/pi-subagents
- A minimal, in-process sub-agent core. Agents run inside the same pi runtime with no spawned subprocesses, and the package exposes a typed API and lifecycle events for other extensions.
- Hard fork of `tintinweb/pi-subagents`, now independently maintained by Chris Lasher.
- Latest confirmed: v25.0.5, published Oct 28, 2026. About 16.6K downloads/month, 1.3 MB, 0 dependencies.
- Deliberately left out of the core: scheduling, cross-extension RPC, model-scope enforcement, a tool denylist, agent memory, and skill preloading.
- Delegated to companion packages: permission policy (`@gotgenes/pi-permission-system`) and git worktree isolation (`@gotgenes/pi-subagents-worktrees`).
- Tools: `subagent`, `get_subagent_result`, `steer_subagent`. Commands: `/subagents:settings`, `/subagents:sessions`.
- Notable features: background concurrency with queueing (default 4), mid-run steering, session resume, `ask_parent` / `notify_parent`, hard turn budgets with wrap-up warnings, context inheritance, fuzzy model names, `locked` frontmatter fields, a live widget, and a transcript viewer.
- Extension surface: typed `SubagentsService`, lifecycle events (`subagents:created/started/completed/failed/resuming/resumed/steered/compacted`), a `WorkspaceProvider` seam, and a layered settings loader.

### nicobailon/pi-subagents (npm: `pi-subagents`)
- A full delegation and workflow package. Foreground children run in-process; background children run in a detached runner.
- Latest confirmed: v0.88.1, published Nov 13, 2026. About 972.8K downloads/month (221.7K/week), 10.7 MB, 5 dependencies. Needs Pi 1.2.0+.
- Built-in agents: `scout`, `researcher`, `evidence-auditor`, `worker`, `reviewer` (read-only), `oracle`, `delegate`.
- Recommended implementation loop: clarify, scout, worker, fresh reviewers, worker.
- Packaged shortcuts: `/parallel-review`, `/review-loop`, `/council`, plus saved `.chain.md` / `.chain.json` workflows.
- Also: background runs, a persistent FleetView with `/subagents-fleet`, `/subagents-doctor`, missions and schedules, worktree isolation, acceptance gates, an opt-in watchdog (adversarial change reviewer, scope monitoring, LSP checks), per-agent persistent memory, agent-local skill paths, and a spawn budget (`maxSubagentSpawnsPerRun`, default 64).
- Extension surface: RPC, a delegation API, preflight, capability ceilings, trusted workflow resources, and background-work providers.
- Some models start with a small `subagents_enable` loader instead of the full `subagent` tool. The `toolActivation` setting changes this.

---

## 3. Head-to-head

| Dimension | @gotgenes | nicobailon |
|---|---|---|
| Philosophy | Minimal composable core | Batteries included |
| Execution | All in-process | Foreground in-process, background detached |
| Orchestration | Primitives; parent LLM sequences | Chains, parallel, saved workflows, review loops |
| Built-in roles | Generic types (verify exact defaults) | scout, worker, reviewer, oracle, and more |
| Custom agents | `.pi/agents/*.md`; model, thinking, tools, `locked` | Same location; adds skills, skillPath, extensions, output, memory |
| Permissions | Via `pi-permission-system` (allow/ask/deny, ask forwarded to parent) | Watchdog, child tool permissions, capability ceilings |
| Isolation | Companion worktree package | Built-in worktree option |
| Memory | Removed from core | Per-agent persistent memory |
| Extensibility | Typed service, events, provider seams | RPC, delegation API, preflight, providers |
| Observability | Widget, transcript viewer | FleetView, run artifacts |
| Adoption | Small | Very large; active releases |

**Verdict:** nicobailon wins on capabilities. Extensibility is roughly a tie: gotgenes is cleaner and more typed, nicobailon is broader.

**Choose gotgenes instead if:** enforced per-agent permissions matter more to you than built-in chains and review loops, or you want a small, auditable core to build your own orchestration on.

---

## 4. `/plan`, `/build`, and custom primary agents

| Option | What it is | Notes |
|---|---|---|
| `@janvitos/pi-plan-build` | Persistent Plan/Build modes, `/plan`, `/build`, Tab cycling, saved plan file, approval, clean-session handoff, per-mode model/thinking memory | Plan-mode bash safety is guidance, not hard enforcement. Only two modes. |
| `MachineLearning-Nerd/pi-plan-build-mode` | Read-only planning, confirmation gate before Build, `propose_plan_change` for scope changes | Stricter gating; `/mode` selector |
| `pi-agent-mode` (ZGltYQ) | Markdown-defined primary agents, `/agent`, Ctrl+Shift+M cycling, `set_agent` tool, default agent in settings, `--agent` flag | Primary agents only; pairs cleanly with either subagent package |
| `pi-open-agents` (andrea-tomassi) | Unified primary + subagent management with `mode: primary \| subagent \| all`, per-agent model/thinking/permissions, OpenCode-format files | Ships its own `subagent` tool that spawns child pi processes, so it overlaps with both candidates. New (0.1.34, listing dated Nov 15). Use it instead of a subagent package, not alongside one. |

Reference model: OpenCode's built-in Build and Plan primary agents, switched with Tab, with Plan denying edits except to plan files.

---

## 5. Recommended architecture

```
Primary agents (you switch between them)
  plan        read-only; investigates, writes the plan, asks questions
  build       executes; carries the orchestration policy
  (optional) orchestrate   a distinct orchestrator persona

Subagents (delegated)
  scout       recon
  planner     returns a structured plan to plan/orchestrator
  worker      implements
  reviewer    fresh-context, read-only
  verifier    custom: can run tests/linters/builds, cannot edit source
  oracle      second opinion on risky decisions
```

Key decisions:
1. **Orchestrator = a primary agent, not a nested subagent.** nicobailon's own skill guidance says the parent owns delegation and review fan-out, and children should not launch subagents or decide the loop outcome.
2. **Verifier is custom.** Neither package ships one. Give it bash and no edit tools. Check whether acceptance gates fit your test commands.
3. **Reviewers use fresh context.** A model reviewing its own work in the same context tends to approve it.
4. **Choose one mode system.** Two "mode" extensions would compete over tool sets.

---

## 6. Things to verify before committing

- [ ] **Shared agent directories.** `pi-agent-mode` and `pi-subagents` both read `~/.pi/agent/agents/` and `.pi/agents/`. Confirm a primary-only agent does not appear as a delegable subagent, and the reverse. Test by asking: "Show me the available subagents."
- [ ] **Tool name collision.** `pi-open-agents` and `pi-subagents` both register a tool named `subagent`. Install only one engine.
- [ ] **Enforcement.** For the `plan` primary and `reviewer`/`verifier` subagents, confirm read-only is enforced by the runtime and not only by prompt guidance.
- [ ] **Delegation activation.** Decide whether you want `toolActivation` left at default or delegation always on.
- [ ] **Nested spawning.** If you still want an Orchestrator subagent, check the recursion guard and spawn-budget settings in nicobailon's workflows and configuration docs.
- [ ] **gotgenes defaults.** Confirm its default agent types and whether `pi-permission-system` rules apply to primary agents or only to children.

---

## 7. Other options noticed

- `pi-agents-flow`: supervisor-led multi-agent orchestration with durable workflow graphs and quality gates, built on a nicobailon fork. Worth a look if you want DAG-style workflows.
- `pi-dynamic-workflows` and the Atomic Workflow SDK: scripted or TypeScript pipelines with review and human gates, discussed in community comparisons.
- `@davecodes/pi-subagents`: a nicobailon fork adding background-work promotion and a live agents browser.

---

## 8. Sources

- https://pi.dev/packages/pi-subagents (nicobailon package page)
- https://pi.dev/packages/@gotgenes/pi-subagents (gotgenes package page)
- https://www.npmjs.com/package/@gotgenes/pi-subagents
- https://github.com/gotgenes/pi-packages (monorepo)
- https://github.com/nicobailon/pi-subagents
- https://pi.dev/packages/pi-open-agents
- https://github.com/ZGltYQ/agent-mode
- https://github.com/janvitos/pi-plan-build
- https://github.com/MachineLearning-Nerd/pi-plan-build-mode
- https://opencode.ai/docs/agents (OpenCode primary-agent model)
- https://mer.vin/2026/05/agent-harnesses-compared-claude-code-workflows-pi-subagents-and-atomic/
