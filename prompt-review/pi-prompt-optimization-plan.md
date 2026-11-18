# Pi prompt optimization: sequential review in one session

Updated: 2026-11-18. Target: the existing interactive Pi 1.5.2 installation.

## Implementation state

- `promptsmith-settings.json` prepares execution-contract rewrites, family `auto`, the active enhancer model, balanced strength, preview ON, auto-send OFF, code preservation ON, conversation/project metadata OFF, and a 45-second timeout.
- `extensions/prompt-review-guard.ts` loads `prompt-review/guard.mjs` through normal Pi extension discovery. The adapter uses the pinned Pi 1.5.2 SDK, matching the existing local workflow adapters.
- **Both optimizers were installed by the owner:** Promptsmith 0.4.0 and Prompt Optimize 0.1.4, each registered once. This implementation did not install, update, remove, or register them. Existing package settings and the staged managed workflow were preserved.
- Promptsmith settings follow its published version-1 schema. The guard supports Prompt Optimize **0.1.4**, its npm entry point, and its documented phase records. Other versions or tool owners are denied during optimization until their contract is reviewed.

## Workflow

Use one ordinary Pi TUI session, without automatic chaining:

1. Write a rough coding request in the editor without submitting it.
2. Invoke Promptsmith to structure it into an execution contract. **Alt+P** preserves the nonempty draft. Replacing that draft by typing `/promptsmith` would lose the input passed to the extension; use the slash command only when your command UI preserves the editor text. Review the preview and accept its replacement into the editor.
3. Copy the complete structured draft before entering another command. Run `/prompt-optimize <structured draft>`. For long text, run `/prompt-optimize`, then paste the draft when asked. Upstream does not automatically consume the editor buffer for this command.
4. Review the complete proposal. **Modify** requests another proposal. **Cancel** rejects it. **Accept** finishes optimization, returns to the recorded branch start, and inserts the accepted prompt into the editor.
5. Inspect the resulting editor text and **manually submit** only when satisfied.

Keep the copied structured draft until the second stage finishes. Do not invoke Promptsmith during Prompt Optimize's phase, begin optimization while coding is running, or queue a coding request behind the phase. Messages during optimization refine the prompt rather than authorize implementation.

Both passes are available; skip an unnecessary pass. Promptsmith normally uses one rewrite request and can retry once for formatting. Prompt Optimize can use several turns. No token or cost savings are guaranteed or measured.

## Owner installation and first use

The owner has installed the official packages separately. Their installed manifests and versions were checked: Promptsmith 0.4.0 and Prompt Optimize 0.1.4. Do not substitute the similarly named `pi-optimize-prompt` package.

After installation, restart Pi, inspect slash-command availability, and run `/promptsmith status`. Confirm preview ON, auto-send OFF, execution-contract, and active enhancer routing. `/promptsmith undo` restores its last pre-enhancement draft; it is not a cross-extension undo history. The existing `/prompt` command remains the saved-template manager.

Changing or resetting Promptsmith settings can disable preview or enable submission. Recheck status afterward. This is a reviewed configuration, not a tamper-proof override of its settings UI.

## Optimization-phase guard

Upstream Prompt Optimize retains existing tools and adds its proposal tool. The local guard adds a temporary restriction through Pi's `tool_call` gate. It neither forks the optimizer nor registers duplicate commands.

During an active phase it permits only:

- `propose-prompt-optimization`, owned by the installed supported optimizer.
- Pi's native `read`, targeting the exact installed `skills/writing-for-agents/SKILL.md`. Unrelated files, traversal, and replacement read implementations are denied.

Every other model tool call is blocked, including Bash, writes, edits, delegation, MCP, codemode, tool discovery, and compression tools. Ponytail, billion-context, RTK, and permission modes remain installed; their lifecycle hooks can still run. Existing permissions apply in addition to this restriction.

The guard replays the current branch's `pi-prompt-optimize` start/end entries at startup, branch changes, before runs, at settlement, and before tool calls. An end record alone does not unlock coding tools: upstream restores them before the proposal call finishes. Restrictions last until Pi is idle and has returned to the recorded branch start. Malformed state stays locked. A new session starts with its own state.

The guard never changes the active tool list, adds a model request, registers a model tool, or injects system instructions. It shows a phase status in the UI. It controls model tool calls, not arbitrary extension code, direct extension side effects, Pi's built-in commands, or OS access. It is not a sandbox. Drafts are sent to the selected provider; do not supply secret content.

## Cancellation and recovery

- Cancel or `/prompt-optimize end` finishes without inserting the rejected proposal. Recover the structured draft from the manual copy.
- Failed return navigation keeps tools blocked. Try `/prompt-optimize end`; if recovery fails, start a fresh session.
- Reload/resume during active or finishing phases restores restrictions. Invalid state requires a fresh session; there is no force-unlock command.
- Missing or unsupported optimizer files do not bypass enforcement. Without an optimization phase, ordinary coding receives no extra restriction.

## Verification

Completed offline checks:

- `node prompt-review/guard.test.mjs` — 10 tests passed.
- `node prompt-review/installed.test.mjs` — 4 tests passed against the installed package source. Promptsmith's actual parser accepts and restores the prepared settings unchanged. Prompt Optimize's actual handlers pass Accept, Modify, Cancel, and explicit end checks using mock UI/session contexts; only its kickoff message is sent, never a coding task.
- A temporary fixture with guard registration removed fails the denial assertions as expected.
- The installed Pi extension loader loads `prompt-review-guard.ts` with zero errors without opening a live model session.

Tests cover normal coding, permitted proposal/skill reads, coding/MCP/compression denial, traversal, tool ownership and version checks, active/finishing phase recovery, finalization siblings, idle navigation, failed navigation, unrelated branches, malformed state, and fresh sessions.

**NOT TESTED:** live Promptsmith preview/undo UI, live Accept/Modify/Cancel, TUI command availability, model transport, composition with billion-context and Ponytail hooks, or the final two-pass TUI workflow. These require a disposable smoke session. Offline source checks with mock contexts do not establish full integration.

Acceptance smoke scenarios after installation: optimize a harmless draft, cancel both stages, undo stage one, modify then accept stage two, explicitly end a phase, and reload mid-phase. Verify the accepted prompt sits in the editor without starting coding. Use a stub provider for attempted disallowed tools and timeout/error cases. Confirm normal tools and existing permissions afterward.

## Rollback

The original document is backed up under `backups/prompt-optimization-*`. With Pi stopped, move the guard adapter outside `extensions/` to disable discovery and preserve or remove only this setup's Promptsmith settings. Do not use conversational optimization with unrestricted coding tools afterward. Package installation/removal remains an owner action. Restore only intended files; avoid overwriting newer changes. No authentication files are backed up.

## References

- [Promptsmith README](https://github.com/ayagmar/pi-promptsmith)
- [Settings schema](https://github.com/ayagmar/pi-promptsmith/blob/master/src/types.ts) and [defaults](https://github.com/ayagmar/pi-promptsmith/blob/master/src/constants.ts)
- [Prompt Optimize source](https://github.com/wicii2120/pi-prompt-optimize/blob/main/extensions/prompt-optimize.ts)
- [Pi extension API](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/extensions.md)

Upstream behavior was researched with Exa and Context7; installed Pi source supplied local API and tool-provenance checks.
