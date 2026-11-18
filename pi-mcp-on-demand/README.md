# pi-mcp-on-demand

**Installed in the active Pi agent directory with activation disabled.** The separate `mcp-on-demand.json` explicitly denies all four inventoried servers (`context7`, `keenable`, `playwright`, `zotero`). It contains no connection definitions or credentials. Native MCP settings are unchanged, and no live server was contacted. This is the contract's initial deny-all configuration, not an enabled live-server setup.

## Configuration

- The extension reads policy from `mcp-on-demand.json` in Pi's active agent directory (`PI_CODING_AGENT_DIR` when set). Version 1 rejects unknown keys; there is no project policy override.
- [`mcp-on-demand.example.json`](mcp-on-demand.example.json) shows the initial disabled, deny-all policy. For a fresh installation, copy the example to `mcp-on-demand.json` in the active agent directory.
- A `deny` rule has `action` and `aliases` only. An `allow` or `ask` rule also needs `roles: ["parent"]`, `definitionSha256` (SHA-256 of `JSON.stringify` of the reviewed effective native config, preserving property order), and `tools` (exact MCP tool names with `allow`, `ask`, or `deny`). Omitted tools are hidden and denied. `allow` at the tool level additionally requires read-only and closed-world annotations; `ask` uses a single-call confirmation. Neither decision replaces the installed permission gate.
- Pi 1.5.2 loads the active agent directory's `mcp.json`, then trusted `.pi/mcp.json`. Full project definitions replace global definitions; partial entries change only `enabled`, `exposure`, or `toolExposure`. Each definition and override must explicitly have `"enabled": false`; the extension never writes these files. Repository-root `.mcp.json` is not a native Pi input.
- Use `[use-mcp:<name>]` at the start of an eligible prompt, or start with a direct request such as `Use <alias> to …`. A leading sequence of directives can authorize multiple servers, within the policy limit. Expanded skill/template text, quoted examples, extension-originated messages, and ambiguous requests do not grant access. `[no-mcp]` suppresses activation.
- Runs use separate, generation-scoped aliases. Policy, native definitions, trust, manager, permission-extension identity, and owned registrations are rechecked before granting tools and on every on-demand call, including after asynchronous approvals. Changes revoke the run; calls do not write configuration. A session transition resets lifecycle state only after cleanup. Steer/follow-up input revokes and queues cleanup rather than extending a grant.
- Check `/mcp-ondemand status`, `/mcp-ondemand policy`, or `/mcp-ondemand audit`. A command-name collision blocks activation. Native `/mcp` Enable/Disable actions persist configuration and are not part of this workflow.

## Compatibility and limits

| Component | Use and limit |
|---|---|
| Pi 1.5.2 native MCP | Required; an alternative `/mcp` implementation or another runtime MCP registration prevents on-demand activation. Uses the installed native loader; re-verify on upgrade. |
| `pi-permission-modes` 2.7.2 | Required; both its `tool_call` gate and this extension's gate apply. A native stdio MCP process is **not** sandboxed by these tool-call checks. Only approve a server whose executable, arguments and startup behavior you trust. |
| Native `codemode` and `tool_search` | The needed discovery tool must already be active; this extension does not edit settings to turn it on. Shared MCP resource operations are denied while an on-demand registration exists. |
| HTTP authorization | Pi treats HTTP definitions without `Authorization` as eligible for OAuth. Version 1 denies implicit/explicit OAuth and provider-token routes. The current Context7 and Keenable definitions are unsupported as configured. No workaround or credential migration is included. |
| `pi-subagents` 0.89.0 | Parent-only (`tui`/`rpc`), with child-process markers rejected. No grant is persisted or forwarded. Foreground ambient loading and background MCP selector preflight are unsupported activation routes. |
| Setup cancellation | A live `ctx.signal` is required before registration. Pi 1.5.2 may not supply it in `before_agent_start`; those paths are refused rather than treated as abort-safe. |
| Teardown | Unregisters only owned aliases. The SDK returns no disconnect-completion promise; registry withdrawal is not proof of process exit. Native tools may remain hidden after disconnect. A cleanup failure blocks further activation and calls; restart Pi. |

## Deployment and rollback

Installed under `/var/home/ahsan/.pi/agent`, the active agent directory at installation. The source resolves policy through Pi's `getAgentDir()`, honoring `PI_CODING_AGENT_DIR`; a future change of agent directory requires a separate installation there. A loaded Pi extension executes as the Pi user without a separate sandbox, can read policy/native definitions and register MCP processes, and installs always-loaded lifecycle/tool hooks and one slash command with three subcommands. The source registers no model-callable tools and makes no extra model request for prompt classification. Startup, inactive-prompt, and token overhead are **not measured** for this revision; zero total overhead is not promised.

Added only `extensions/pi-mcp-on-demand/{index,match,policy,registry}.ts` and `mcp-on-demand.json` in the active agent directory. Every destination was absent; exclusive copies prevented overwriting existing files. At installation, the installed source and policy matched the staged versions byte for byte, and native `mcp.json` and `settings.json` were unchanged. Permission settings and agent roles were not edited. Enabling or granting any server is a separate review and approval, not an installation side effect.

To load the installed extension, use `/reload` in Pi or restart Pi. It will be discovered through the SDK-supported `extensions/*/index.ts` layout without a settings change. Then use the read-only commands:

```text
/mcp-ondemand status
/mcp-ondemand policy
/mcp-ondemand audit
```

The `audit` command reads native definitions through the installed loader; it does not register or connect a server. The extension has not been loaded or exercised as part of installation. Do not set `enabled` to `true` merely to make a prompt work: named rules still deny access, capabilities remain unreviewed, and the cancellation/authentication limits above still apply.

For rollback, move only the added `extensions/pi-mcp-on-demand/` directory and active `mcp-on-demand.json` out of their discovery locations, then `/reload` or restart. Preserve the staged source and unrelated extensions. No native MCP configuration repair is needed. Before editing an existing file, retain an operator-approved backup outside discovery; session scratch backups are temporary, not a durable rollback archive.

## Current deployment state

The separate policy is installed disabled and all four named rules deny access; no fingerprints or capabilities have been invented. Installation is confirmed by destination-file presence and byte-for-byte source/policy comparison. Final-revision runtime behavior and production readiness are **NOT VERIFIED**; no live MCP connection is authorized by this work.

The native global `mcp.json` SHA-256 is `d0eb8606b292123f31d9e60b087ad3bf58fcc33cfa49ded58c96808dd77f214d`; it has not been edited. No `.pi/mcp.json` is present in this workspace. The non-native root `.mcp.json` is outside the native configuration inventory and was not inspected.
