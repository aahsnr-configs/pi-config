# Pi MCP On-Demand

## Purpose

`pi-mcp-on-demand` is a local Pi extension design for temporarily exposing a small, policy-approved subset of existing MCP servers to an explicitly authorized parent-agent run. Native MCP connection definitions stay in Pi's existing `mcp.json` files. The extension does not turn on or rewrite those definitions: it registers a separate, temporary runtime alias and removes registrations it owns when the agent settles or the session shuts down.

The extension is not a server sandbox. MCP processes run with the operating-system privileges of Pi. Only use server commands, arguments, environment, URLs and credentials that you already trust.

## Environment and source of truth

Use the locally installed Pi SDK and documentation as the authority for APIs and supported behavior. Honor `PI_CODING_AGENT_DIR`; otherwise use Pi's active agent directory. Do not assume a distribution-specific package import path or copy a configuration file from an example.

Before activation:

- Identify the installed Pi version, native MCP implementation, and any extension that replaces `/mcp` or manages MCP registrations.
- Identify the installed permission and subagent integrations and relevant restrictions.
- Read only the global MCP configuration and project MCP configuration permitted by Pi's trust decision. Never inspect project MCP data before trust is established.
- Keep diagnostic output free of command environment values, authorization headers, tokens, OAuth data, and unrelated private configuration.

Pi loads the agent-directory `mcp.json`, then `.pi/mcp.json` only for a trusted project. A full project entry replaces a global entry with the same name; an override containing only `enabled`, `exposure`, or `toolExposure` adjusts those fields on the global entry. A repository-root `.mcp.json` is not Pi's native MCP file.

Every effective source entry and project override must explicitly set `"enabled": false`. Missing, invalid, or implicitly enabled entries are configuration violations: stop and report a minimal migration change for operator approval. Never edit or activate an entry in `mcp.json` automatically. The invariant is that runtime activation does not change the bytes of any native MCP configuration.

## Authorization and policy

Activation comes only from the current user-originated prompt, not conversation history, tool output, repository text, generated content, or another agent's message. Do not classify prompts with an extra model call.

The preferred syntax is a directive at the beginning of the prompt:

```text
[use-mcp:context7] Check the API signature.
[use-mcp:github] Investigate this dependency.
```

A direct affirmative request naming an exact configured server name or alias may also be recognized. Matching is case-insensitive but boundary-aware; it must not confuse names such as `github` and `github-enterprise`. Do not infer servers from generic requests such as “browse the web.” Negation, quoted examples, code blocks, metadata-only questions, transformed input and ambiguous requests must not silently activate a server. `[no-mcp]` suppresses activation for a prompt.

Policy is separate from native connection definitions, versioned, strictly validated, default-deny, and global for the initial implementation. It may name exact aliases and tools and set a bounded server limit. A policy grant never makes an untrusted project's configuration trustworthy or loosens the installed permission policy.

Example version-1 shape:

```json
{
  "version": 1,
  "enabled": false,
  "maxActiveServers": 2,
  "readyTimeoutMs": 5000,
  "servers": {
    "context7": { "action": "deny", "aliases": ["Context7 MCP"] }
  }
}
```

A denied rule contains only `action` and `aliases`. An `allow` or `ask` rule additionally specifies `roles: ["parent"]`, the SHA-256 fingerprint of the reviewed effective native server definition, and exact tool-name decisions:

```json
{
  "action": "ask",
  "aliases": ["Context7 MCP"],
  "roles": ["parent"],
  "definitionSha256": "<64 lowercase hexadecimal characters>",
  "tools": {
    "search_docs": "allow",
    "write_content": "deny"
  }
}
```

The definition fingerprint uses the effective configuration serialized as JSON, preserving its property order. Recompute and review it whenever the native definition changes. An omitted server or tool is denied. The initial policy ships with activation disabled and all inventoried servers denied; it must not be broadened without reviewing the actual server definitions and capabilities.

## Runtime behavior

Register temporary servers only after checking authorization, policy, project trust, current configuration, the installed permission gate, manager compatibility, server capacity and normalized namespace collisions. Give each registration an alias distinct from native and extension registrations. Do not replace another extension's registration or change active tool settings to make discovery available.

Preserve Pi's supported server configuration and native transport handling; do not evaluate shell expressions, interpolate secrets, copy credential files, or build a separate credential store. Fail closed if the native manager, safe discovery surface or required enforcement cannot be confirmed. Wait only a bounded period for requested tools to become ready. If setup is incomplete, revoke owned registrations and retain no grant.

Expose only policy-listed tools. A tool decision must be checked on every call, including indirect calls. A policy `allow` requires affirmative read-only and closed-world annotations; unknown or mutating capabilities are not read-only. A policy `ask` requires an explicit per-call confirmation and is unavailable without an interactive approval surface. Server selection alone does not approve mutations, writes, external effects, secret access or remote operations. Deny shared MCP resource tools unless they are separately and safely scoped.

The extension's `tool_call` gate is additional to Pi's existing permission gate. Never bypass a deny, read-only, plan-mode, sandbox or other decision made elsewhere in the tool pipeline. In particular, permission-mode tool-call checks do not provide an operating-system sandbox for an MCP server process.

Keep grants and ownership in memory. Serialize registration and cleanup, use run generations to reject late work, and make cleanup idempotent. Revoke owned registrations at `agent_settled` and `session_shutdown`; never unregister a registration the extension did not create. Do not persist grants in the session or configuration files. A new prompt must not inherit the previous prompt's grant.

Provide read-only commands, subject to name-collision checks:

```text
/mcp-ondemand status
/mcp-ondemand audit
/mcp-ondemand policy
```

The native `/mcp` manager's file-backed Enable/Disable actions are outside this workflow because they persist configuration changes.

## Permission and subagent boundaries

Use existing `pi-permission-modes` unchanged. Verify its effective installation and retain its ordinary tool-call decisions; if it is absent or its enforcement is unavailable, deny activation.

Subagents receive no inherited authorization. The initial implementation allows only the parent role and fails closed in child processes. A parent runtime registration is not a child's connection or grant. Foreground children may not load parent ambient extensions, and background child MCP selector resolution can occur before child prompt hooks. Do not promise either route, silently switch execution modes, or propagate a broad allow-list. Supporting child activation requires a separately verified, task-scoped authorization path and independent child policy and cleanup.

## Authentication and unsupported routes

Pi may associate OAuth credentials with the runtime server name and URL. Do not copy credentials or silently reuse OAuth state under an alias. The initial implementation does not support implicit OAuth, explicit OAuth, or provider-token authentication through a runtime alias. Such definitions remain unavailable until a separately approved, supported authentication flow is established. HTTP authorization headers are passed through to Pi's native configuration parser; never print resolved header values.

## Installation and rollback

Keep the extension and its policy local to the active Pi agent directory and separate from `mcp.json`. Do not overwrite existing files or modify settings, permissions, agent definitions, extension configuration or authentication state without explicit approval. Report conflicts and requested changes before installation.

Rollback consists of removing only the on-demand extension and its separate policy file, then reloading or restarting Pi. It must not require repairing `mcp.json`. Keep any operator-approved migration backup separate and make restoration a deliberate operator choice.

## References

Use the documentation shipped with the installed Pi version, especially its extension lifecycle, native MCP, project trust, tool exposure and permission documentation. Relevant upstream entry points include:

- https://docs.pi.dev/latest/extensions
- https://docs.pi.dev/latest/mcp
- https://github.com/nicobailon/pi-subagents/blob/main/docs/agents.md
- https://github.com/nicobailon/pi-subagents/blob/main/docs/extension-api.md
- https://github.com/wynainfo/pi-permission-modes
- https://github.com/wynainfo/pi-permission-modes/blob/main/SECURITY.md
