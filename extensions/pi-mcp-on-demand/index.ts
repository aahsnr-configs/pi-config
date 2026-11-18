import { realpathSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { setTimeout as sleep } from 'node:timers/promises';
import { getAgentDir, getPackageDir, VERSION, type ExtensionAPI, type ExtensionContext, type McpServerConfig } from '@earendil-works/pi-coding-agent';
import { resolvePrompt } from './match.ts';
import { parsePolicy, type Policy, type Rule } from './policy.ts';
import { fingerprint, readJson, registry, supportedTransport, type NativeRegistry } from './registry.ts';

const command = 'mcp-ondemand';
const toolNamespace = (name: string) => `mcp__${name.replace(/-/g, '_')}`;
const candidate = /^\s*(?:\[use-mcp:|(?:please[ ,]+)?(?:use|connect to) +)/i;
const resources = new Set(['list_mcp_resources', 'list_mcp_resource_templates', 'read_mcp_resource']);
const builtin = (path: string) => `builtin:${path}`;
const note = (ctx: ExtensionContext, text: string, level: 'info' | 'warning' = 'warning') => ctx.ui.notify(`mcp-on-demand: ${text}`, level);
const blocked = { block: true, reason: 'mcp-on-demand: no current grant for this capability' } as const;
const isChild = () => Boolean(process.env.PI_SUBAGENT_CHILD || process.env.PI_SUBAGENT_PARENT_SESSION);
const policyStamp = (policy: Policy) => fingerprint({
  ...policy,
  servers: [...policy.servers].map(([key, rule]) => [key, rule.action === 'deny' ? rule : { ...rule, tools: [...rule.tools] }]),
});

type Owned = { name: string; alias: string; tools: Map<string, 'allow' | 'ask' | 'deny'>; config: McpServerConfig };
type Run = {
  id: number;
  controller: AbortController;
  signal: AbortSignal;
  grants: Map<string, Owned>;
  cwd: string;
  trusted: boolean;
  boundary?: { policy: string; registry: string };
};

export default function extension(pi: ExtensionAPI) {
  let pendingInput: string | undefined;
  let dirtyInput = false;
  let generation = 0;
  let run: Run | undefined;
  let closed = false;
  let queue = Promise.resolve();
  const owned = new Map<string, Owned>();
  const readPolicy = () => parsePolicy(readJson(join(getAgentDir(), 'mcp-on-demand.json')));
  let sdk: Promise<NativeRegistry> | undefined;
  const native = () => sdk ??= import(pathToFileURL(join(getPackageDir(), 'dist/extensions/mcp/config.js')).href) as Promise<NativeRegistry>;
  const enqueue = (work: () => void | Promise<void>) => {
    const next = queue.then(work);
    queue = next.catch(() => undefined);
    return next;
  };

  const verifyManager = () => {
    const commands = pi.getCommands();
    const mcp = commands.filter(c => c.name === 'mcp');
    return VERSION === '1.5.2' && mcp.length === 1 && mcp[0].sourceInfo.path === builtin('mcp');
  };
  const verifyCommand = () => {
    const commands = pi.getCommands().filter(c => c.name === command || c.name.startsWith(`${command}:`));
    return commands.length === 1 && commands[0].name === command;
  };
  const verifyPermissionGate = () => {
    const commands = pi.getCommands().filter(c => c.name === 'perm');
    if (commands.length !== 1 || commands[0].source !== 'extension') return false;
    try {
      const entry = realpathSync(commands[0].sourceInfo.path);
      const pkg = readJson(join(dirname(entry), '..', 'package.json'));
      return entry.endsWith('/src/index.ts') && pkg?.name === 'pi-permission-modes' && pkg.version === '2.7.2';
    } catch { return false; }
  };
  const parent = (ctx: ExtensionContext) => !isChild() && (ctx.mode === 'tui' || ctx.mode === 'rpc');
  const current = (r: Run, ctx: ExtensionContext) => run === r && !r.signal.aborted && !ctx.signal?.aborted &&
    !closed && r.cwd === ctx.cwd && r.trusted === ctx.isProjectTrusted() && parent(ctx);
  const revoke = () => {
    generation++;
    run?.controller.abort();
    run = undefined;
  };
  const cleanup = (ctx: ExtensionContext) => {
    for (const [alias] of owned) {
      try {
        pi.unregisterMcpServer(alias);
        if (pi.getMcpServers().some(entry => entry.name === alias)) throw new Error('Registration remains');
        owned.delete(alias);
      } catch { note(ctx, 'Could not remove an owned registration. Its calls are blocked; restart Pi.'); }
    }
  };
  const clearInput = () => { pendingInput = undefined; dirtyInput = false; };
  const settle = (ctx: ExtensionContext) => {
    clearInput();
    revoke();
    return enqueue(() => cleanup(ctx));
  };
  const load = async (ctx: ExtensionContext) => {
    if (!verifyManager()) throw new Error('Native MCP manager unavailable or unverified Pi version');
    const api = await native();
    const effective = registry({ agentDir: getAgentDir(), cwd: ctx.cwd, projectTrusted: ctx.isProjectTrusted() }, api);
    return { api, effective };
  };
  const toolFor = (name: string, alias: string) => pi.getAllTools().find(tool =>
    tool.name === name && tool.namespace?.name === toolNamespace(alias) &&
    tool.sourceInfo.path === builtin('mcp') && tool.exposure !== 'hidden');
  const discoveryReady = (name: string) => pi.getActiveTools().includes(name) && pi.getAllTools().some(t =>
    t.name === name && t.sourceInfo.path === builtin(name === 'tool_search' ? 'tool-search' : name));
  const namespaceOccupied = (alias: string) => pi.getAllTools().some(tool =>
    tool.namespace?.name === toolNamespace(alias) || tool.name.startsWith(`${toolNamespace(alias)}__`));
  const boundaryValid = async (r: Run, ctx: ExtensionContext) => {
    try {
      if (!current(r, ctx) || !r.boundary || !verifyCommand() || !verifyPermissionGate()) return false;
      const { effective } = await load(ctx);
      const policy = readPolicy();
      const registrations = pi.getMcpServers();
      return current(r, ctx) && verifyCommand() && verifyPermissionGate() && policy.enabled &&
        r.boundary.policy === policyStamp(policy) && r.boundary.registry === fingerprint(effective) &&
        registrations.length === owned.size && [...owned.values()].every(entry =>
          registrations.some(server => server.name === entry.alias && fingerprint(server.config) === fingerprint(entry.config)));
    } catch { return false; }
  };

  pi.on('session_start', async (_event, ctx) => {
    await settle(ctx);
    closed = false;
    if (!verifyCommand()) note(ctx, 'Command collision; on-demand activation blocked.');
  });

  pi.on('input', async (event, ctx) => {
    // Steers/follow-ups can change the task mid-run but may never extend its grant.
    const hadRun = Boolean(run);
    if (hadRun) revoke();
    if (!parent(ctx) || event.source === 'extension' || event.streamingBehavior || !candidate.test(event.text)) {
      clearInput();
    } else if (pendingInput !== undefined || dirtyInput) {
      pendingInput = undefined;
      dirtyInput = true;
    } else {
      pendingInput = event.text;
    }
    if (hadRun) await enqueue(() => cleanup(ctx));
  });

  pi.on('before_agent_start', async (event, ctx) => {
    const input = pendingInput;
    clearInput();
    // Never authorize text inserted by an input transformer, a skill, or a prompt template.
    if (event.messageType !== 'user' || !input || input !== event.prompt || !parent(ctx) || closed) return;
    revoke();
    const controller = new AbortController();
    const signal = ctx.signal ? AbortSignal.any([controller.signal, ctx.signal]) : controller.signal;
    const r: Run = { id: generation, controller, signal, grants: new Map(), cwd: ctx.cwd, trusted: ctx.isProjectTrusted() };
    run = r;
    await enqueue(async () => {
      if (!current(r, ctx)) return;
      cleanup(ctx);
      const deny = (reason: string) => note(ctx, reason);
      try {
        const policy = readPolicy();
        if (!policy.enabled) return;
        const request = resolvePrompt(event.prompt, policy);
        if (request.kind === 'none') return;
        if (request.kind === 'ambiguous') return deny('Request not recognized; use [use-mcp:<configured-name>] at the start of a new prompt.');
        if (!verifyCommand()) return deny('Command collision; activation refused.');
        if (!ctx.signal || ctx.signal.aborted) return deny('Cancellable setup signal unavailable; activation refused.');
        if (owned.size || !verifyPermissionGate()) return deny('Cleanup or the verified permission gate is unavailable.');
        if (request.names.length > policy.maxActiveServers) return deny('Request exceeds server capacity.');
        const { api, effective } = await load(ctx);
        const registrations = pi.getMcpServers();
        if (registrations.length) return deny('Another runtime MCP registration is present.');
        const occupied = new Set(effective.servers.map(s => toolNamespace(s.name)));
        const bundle: { rule: Rule; entry: Owned }[] = [];
        for (const name of request.names) {
          const rule = policy.servers.get(name);
          const server = effective.servers.find(entry => entry.name === name);
          if (!server || !rule || rule.action === 'deny' || !rule.roles.includes('parent')) return deny('A requested server is not approved.');
          if (fingerprint(server.config) !== rule.definitionSha256) return deny('A requested definition changed or was not reviewed.');
          if (!supportedTransport(server.config)) return deny('OAuth, implicit OAuth and provider authentication are unsupported for runtime aliases.');
          const alias = `ondemand_${name.replace(/[-_]+/g, '_').slice(0, 18)}_${r.id.toString(36)}`;
          const namespace = toolNamespace(alias);
          if (occupied.has(namespace) || namespaceOccupied(alias)) {
            return deny('Runtime MCP namespace collision.');
          }
          occupied.add(namespace);
          const tools: Owned['tools'] = new Map();
          const exposure = Object.create(null) as NonNullable<McpServerConfig['toolExposure']>;
          exposure['*'] = 'hidden';
          const exposures = new Set([server.config.exposure ?? 'codemode']);
          for (const [tool, action] of rule.tools) {
            if (action === 'deny') continue;
            const visible = api.getMcpToolExposure(server.config, tool);
            if (visible === 'hidden') continue;
            const fullName = `${namespace}__${tool}`;
            if (!/^[A-Za-z0-9_]{1,64}$/.test(fullName)) return deny('Unsupported or oversized MCP tool name.');
            exposures.add(visible);
            exposure[tool] = visible;
            tools.set(fullName, action);
          }
          if (!tools.size) return deny('No approved tools for a requested server.');
          if ((exposures.has('codemode') && !discoveryReady('codemode')) ||
              (exposures.has('deferred') && !discoveryReady('tool_search'))) {
            return deny('Required native discovery tool is not already active; no tool settings were changed.');
          }
          bundle.push({ rule, entry: { name, alias, tools, config: { ...server.config, enabled: true, toolExposure: exposure } } });
        }
        for (const { rule, entry } of bundle) {
          if (rule.action === 'ask' && (!ctx.hasUI || !await ctx.ui.confirm('Connect MCP for this task?', `Allow ${entry.name} for this run only?`, { signal, timeout: 30_000 }))) return;
          if (!current(r, ctx)) return;
        }
        // Recheck the config and permission boundary after asynchronous approvals.
        const checked = await load(ctx);
        if (!current(r, ctx) || !verifyCommand() || !verifyPermissionGate() || pi.getMcpServers().length ||
            policyStamp(readPolicy()) !== policyStamp(policy) ||
            fingerprint(checked.effective) !== fingerprint(effective) ||
            bundle.some(({ entry }) => namespaceOccupied(entry.alias))) return deny('Authorization changed before registration.');
        r.boundary = { policy: policyStamp(policy), registry: fingerprint(effective) };
        for (const { entry } of bundle) {
          pi.registerMcpServer(entry.alias, entry.config);
          owned.set(entry.alias, entry);
        }
        const deadline = performance.now() + policy.readyTimeoutMs;
        while (current(r, ctx) && performance.now() < deadline) {
          if (bundle.every(({ entry }) => [...entry.tools.keys()].every(name => toolFor(name, entry.alias)))) {
            if (!await boundaryValid(r, ctx)) return deny('Authorization changed during startup.');
            for (const { entry } of bundle) r.grants.set(entry.alias, entry);
            note(ctx, `Active for this run: ${bundle.map(({ entry }) => entry.name).join(', ')}`, 'info');
            return;
          }
          await sleep(20, undefined, { signal });
        }
        if (current(r, ctx)) deny('Not all approved native tools became ready; registration withdrawn.');
      } catch {
        if (current(r, ctx)) deny('Activation failed (invalid policy, unavailable connection or runtime error).');
      } finally {
        if (!r.grants.size) {
          if (run === r) revoke();
          cleanup(ctx);
        }
      }
    });
  });

  pi.on('tool_call', async (event, ctx) => {
    if (!owned.size) return;
    // Shared resource tools can list every active server and are not capability-allowlisted in v1.
    if (resources.has(event.toolName)) return blocked;
    const entry = [...owned.values()].find(e => event.toolName.startsWith(`${toolNamespace(e.alias)}__`));
    if (!entry) return;
    const r = run;
    if (!r || r.grants.get(entry.alias) !== entry) return blocked;
    if (!await boundaryValid(r, ctx)) {
      if (run === r) {
        revoke();
        await enqueue(() => cleanup(ctx));
      }
      return blocked;
    }
    const action = entry.tools.get(event.toolName);
    const tool = toolFor(event.toolName, entry.alias);
    if (!tool || !action || action === 'deny') return blocked;
    const hints = tool.annotations;
    if (action === 'allow' && hints?.readOnlyHint === true && hints.destructiveHint !== true && hints.openWorldHint === false) return;
    if (action === 'ask' && ctx.hasUI) {
      const signal = ctx.signal ? AbortSignal.any([r.signal, ctx.signal]) : r.signal;
      const approved = await ctx.ui.confirm('Approve one MCP call?', `Allow ${entry.name}/${event.toolName} once?`, { signal, timeout: 30_000 });
      if (!await boundaryValid(r, ctx)) {
        if (run === r) {
          revoke();
          await enqueue(() => cleanup(ctx));
        }
        return blocked;
      }
      const approvedTool = toolFor(event.toolName, entry.alias);
      if (approved && approvedTool && fingerprint(approvedTool) === fingerprint(tool)) return;
    }
    return blocked;
  });

  pi.on('agent_settled', (_event, ctx) => settle(ctx));
  pi.on('session_shutdown', (_event, ctx) => { closed = true; return settle(ctx); });
  pi.registerCommand(command, {
    description: 'Read-only MCP on-demand status, policy or audit',
    handler: async (arg, ctx) => {
      try {
        if (!arg.trim() || arg.trim() === 'status') {
          note(ctx, `${run?.grants.size ?? 0} granted; ${owned.size} owned; native manager ${verifyManager() ? 'verified' : 'unavailable'}; permission gate ${verifyPermissionGate() ? 'identified' : 'unavailable'}.`, 'info');
        } else if (arg.trim() === 'policy') {
          const policy = readPolicy();
          note(ctx, `Policy ${policy.enabled ? 'enabled' : 'disabled'}; maximum ${policy.maxActiveServers}; ${policy.servers.size} server rule(s); parent only.`, 'info');
        } else if (arg.trim() === 'audit') {
          const { effective } = await load(ctx);
          note(ctx, `${effective.servers.length} disabled definition(s); ${effective.servers.filter(s => !supportedTransport(s.config)).length} unsupported auth route(s); project ${ctx.isProjectTrusted() ? 'trusted' : 'ignored'}.`, 'info');
        } else note(ctx, 'Usage: /mcp-ondemand status | audit | policy');
      } catch { note(ctx, 'Audit failed; check the native manager, disabled baseline and version-1 policy.'); }
    },
  });
}
