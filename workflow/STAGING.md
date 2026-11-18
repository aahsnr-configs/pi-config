# Managed workflow staging

For current launch and research instructions, use [`../pi-workflow-config.md`](../pi-workflow-config.md).
The staging results below describe the earlier configuration.

The separate SDK launcher is `launch.py`; it is not installed into `bin/pi` or
live settings. See [`../MANAGED_PI_STAGING.md`](../MANAGED_PI_STAGING.md) for
verification results and the operator-only smoke and patch review steps.

## Entry points and boundaries

- `launch.py` owns the non-queuing per-user lock, creates a private temporary
  agent directory, and launches `sdk-host.mjs` with a restricted environment.
  `terminal.mjs` owns all user input; the stock TUI dispatches some built-ins
  before extension hooks can intercept them and is deliberately not used.
- The SDK host uses the explicit permission-modes and subagents adapters and
  `controller-extension.ts`, with no ambient project trust or resource
  discovery. The controller delegates only over the installed structured
  foreground event bridge. `dispatch.mjs` verifies the installed preflight
  projection, terminal identity, launch digest, and `child-guard.ts` lease
  acknowledgement. A mere guard startup exception is insufficient: Pi logs
  such exceptions and continues. Until a guard acknowledges, the broker
  denies role tools.
- `state.mjs`, `handoffs.mjs`, and `orchestrator.mjs` keep approval, status,
  handoffs, reuse, one repair, and cancellation in memory. Only the host
  broker can attest writes and checks.
- `broker.mjs`, `workspace.py`, and `private_workspace.py` use filtered,
  bounded, descriptor-relative copies, revalidate fingerprints, and produce a
  patch instead of editing the source. `check_runner.py` uses fixed argv,
  a read-only copy and a networkless Bubblewrap sandbox; no host fallback.
- `research.mjs` and `child-mcp.ts` use the installed Pi MCP client directly.
  Evidence roles can request fixed public catalog IDs; the broker connects
  lazily, without credentials, generic MCP tools or codemode. Real endpoint
  compatibility is not assumed.

**Latest offline results:** 27 Node tests, 4 Python tests, launcher help,
and the Bubblewrap profile probe passed. The real SDK foreground bridge ran
against a fake provider, and the installed MCP client ran against stub HTTP
responses. Manifest pins are Pi 1.5.2, pi-subagents 0.89.0 and
pi-permission-modes 2.7.2.

**NOT TESTED:** Bubblewrap execution in a separate operator launch
environment, external provider startup, full launcher contention and lock
drain with isolated descendants, real public MCP calls, and activation.
No live credentials were used and no patch was applied.
