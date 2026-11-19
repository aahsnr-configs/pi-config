# Pi workflow configuration

The managed Pi workflow begins ordinary tasks in read-only Plan mode. Use `/build` to approve a plan and prepare a scoped patch. Use `/orchestrator` when a task needs a sequence of specialist roles. Orchestrator runs only when invoked.

## Start a managed session

Run this command directly against the bundled Python launcher (no `bin/pi` wrapper is included) from the project directory in a terminal with `OPENAI_API_KEY` available through your credential manager. The workflow uses Pi 1.5.2, pi-subagents 0.89.0, and pi-permission-modes 2.7.2.

```bash
pi_session_log=$(mktemp)
/usr/bin/python3 -I "${PI_CODING_AGENT_DIR:-$HOME/.pi/agent}/workflow/launch.py" "$(pwd -P)" \
  openai/gpt-6.1-terra openai/gpt-6-luna,openai/gpt-6.1-sol \
  --advisor openai/gpt-6.1-sol | tee "$pi_session_log"
```

Terra handles the main work, Luna gathers evidence, and Sol is available on request. For other models, use `provider/model` IDs from the installed Pi catalog and separate ingestion pool entries with commas. The launcher uses supported API key environment variables. It requires a canonical, user-owned project directory and a private, writable `/run/user/$(id -u)` directory. Choose a project whose permitted contents can be sent to your model provider. When the `ready` record appears, enter workflow commands one line at a time and wait for each result. [Launcher](launch.py)

## Extensions and permissions

- The controller owns workflow commands. Its pi-permission-modes adapter reserves `/plan` for this workflow; upstream pi-permission-modes uses that command for plan-file approval.
- The parent and child sessions load the required extensions explicitly. The permission baseline blocks native shell and file tools, `show_plan`, background shells, and unlisted extension tools. Each child receives only its role’s allowed tools. The managed session excludes ambient project extensions and generic MCP discovery.
- Pi-subagents uses its structured foreground API. The launcher configures concurrency at `1` and disables asynchronous delegation, nesting, scheduling, and arbitrary dispatch. The controller binds each child to its role, model, workspace, and tool permissions.
- A per-user lock allows only one managed session at a time. A competing launch reports that the workflow is busy. Cancellation and timeout retain the lock until active work has settled.
- Workspace tools use `fd` to discover files, `rg` to search, and `bat` for bounded reads. Accept only validated workspace-relative paths and fixed arguments. Exclude traversal, symlinks, hard links, special files, and sensitive paths such as `.env*`, credentials, keys, auth files, session data, and Pi policy files.

[Permission modes](../npm/node_modules/pi-permission-modes/docs/modes.md) · [Subagent configuration](https://github.com/nicobailon/pi-subagents/blob/main/docs/configuration.md) · [Structured delegation and capability ceilings](https://github.com/nicobailon/pi-subagents/blob/main/docs/extension-api.md)

## Everyday commands

| Command | Behavior |
|---|---|
| Ordinary prompt or `/plan <task>` | Gather information and prepare a read-only plan in memory. Follow-up prompts refine the pending plan. |
| `/plan` | Show the pending plan, its write scope, and its acceptance criteria. |
| `/build` | Approve the current plan. The controller rechecks scope and source fingerprints before allowing edits in the private workspace copy. If inputs drift or scope changes, revise the plan before approving again. Build returns to Plan when finished. |
| `/patch` | Emit the current patch for review. Changes are staged in the launcher's private copy. Applying the patch to the source project is a separate operation. |
| `/status` | Show the current mode, stage, and busy or pending state. |
| `/cancel` | Cancel active work or leave an idle Orchestrator workflow. |
| `/quit` | Exit the managed session. |

Plan denies agent writes through native tools, delegated tools, and subprocesses. Build allows writes only within the approved scope and when the expected source fingerprints still match.

Once `/patch` emits a `patch` record for the current task, enter `/quit`. Run the following in the same terminal to save and read the patch. The supported changes are scoped edits and new UTF-8 text files in existing directories.

```bash
pi_patch=$(mktemp)
jq -R -s -j 'split("\n") | map(fromjson? | select(.type == "patch")) | last | .patch // empty' \
  "$pi_session_log" > "$pi_patch"
bat --language=diff "$pi_patch"
```

After reviewing it, apply it with `git apply "$pi_patch"` from the repository root if you launched the workflow there. If you selected a repository subdirectory as the workspace, run `git apply --directory=<workspace-path-relative-to-repo-root> "$pi_patch"` from the repository root.

## Models and research

Choose the core model at launch. Plan and Build use it for the main task and select the lowest-cost eligible distinct model from the ingestion pool for information gathering. Use low thinking for routine research and medium for harder research or plan refinements when supported. Keep the core model fixed for the task.

To enable an advisor, include it in the model pool and pass `--advisor provider/advisor-model` at launch. Choose a reasoning model distinct from the core and selected ingestion model, and with higher cost than both. Use `/advisor decision <reason>` for a consequential decision, `/advisor conflict <reason>` for conflicting information, or `/advisor blocker <reason>` when progress is blocked. This authorizes one high-thinking advisory call for the next Plan or Build pass.

Add `--research` and a public query, a public HTTPS documentation URL, or both to the launch command. Use `--context7-library` with the public query when you have a Context7 library ID. For example, append these arguments before `| tee` to research React 19.

```bash
--research --public-query 'React 19 useActionState API and pending state' \
  --public-url 'https://react.dev/reference/react/useActionState' \
  --context7-library '/facebook/react/v19.0.0'
```

Keenable and Exa use their public MCP endpoints. Context7 can use `CONTEXT7_API_KEY` from your credential manager for higher limits. Evidence roles choose among the enabled research actions. They cannot supply their own search terms or URLs. The broker bounds responses and reuses successful lookups within the session. Supply only public terms and URLs. [Keenable MCP](https://docs.keenable.ai/mcp-server) · [Exa MCP](https://exa.ai/docs/get-started/exa-mcp) · [Context7 MCP](https://context7.com/docs/resources/all-clients)

## Orchestrator

Use `/orchestrator <task>` to start a workflow. Use `/orchestrator resume` to promote the current Plan or Build task, or continue the active Orchestrator. Use `/orchestrator approve` to authorize the pending Worker scope. Plan and Build commands are inactive while Orchestrator is running.

| Role | Permissions and handoff |
|---|---|
| **Scouter** | Discover, search, and read relevant files. Use approved public research when useful. Pass relevant paths and findings to the Planner. |
| **Planner** | Use scoped reads and Scouter findings to define the work, permitted write paths, and acceptance criteria. |
| **Worker** | Read and edit within the approved scope. Report changed paths and the resulting patch. |
| **Reviewer** | Inspect the scoped change against the plan and report actionable findings. |
| **Verifier** | Compare the candidate and available information with the acceptance criteria, then report the outcome and any remaining gaps. |

Roles run one at a time and receive bounded handoffs. On promotion, the controller reuses completed work when the task, workspace, scope, and relevant fingerprints still match. Changed inputs require fresh approval before stale work is repeated. One approved repair may address new findings within the original scope.

Keep child context fresh, pass only relevant paths and summaries, and bound launches, tool calls, research output, and elapsed time. Use role-level usage information when the provider reports it. Workflow state lives in memory; each new managed session starts in Plan.
