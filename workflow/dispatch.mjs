import { randomUUID } from 'node:crypto';
import { bindRole, roleTools } from './roles.mjs';
import { checkIds, validHandoff } from './handoffs.mjs';
import { here } from './host-policy.mjs';

const runtimeExtension = here('../npm/node_modules/pi-subagents/src/runs/shared/subagent-prompt-runtime.js');

const evidenceRoles = new Set(['scout', 'plan-evidence', 'build-evidence']);
const writers = new Set(['worker', 'build-core']);
const alias = stage => ({ 'plan-evidence': 'scout', 'plan-core': 'planner', 'build-core': 'worker', 'build-evidence': 'reviewer', advisor: 'reviewer' }[stage] ?? stage);
const summary = { type: 'string', minLength: 1, maxLength: 8192 };
const paths = { type: 'array', items: { type: 'string', maxLength: 512 }, maxItems: 32, uniqueItems: true };
const checks = { type: 'array', items: { enum: checkIds }, maxItems: checkIds.length, uniqueItems: true };
const verdict = { enum: ['accept', 'changes-required'] };
const criteria = { type: 'array', items: { type: 'string', minLength: 1, maxLength: 512 }, minItems: 1, maxItems: 12, uniqueItems: true };
const followUp = { scout: { paths }, planner: { paths, checks, criteria }, worker: { paths }, reviewer: { verdict }, verifier: {} };
const schemaFor = stage => {
  const properties = { summary, ...followUp[alias(stage)] };
  return { type: 'object', properties, required: Object.keys(properties), additionalProperties: false };
};

const exact = (left, right) => Array.isArray(left) && left.length === right.length &&
  new Set(left).size === left.length && left.every(value => right.includes(value));
const modelName = model => model?.replace(/:(off|minimal|low|medium|high|xhigh|max)$/u, '');

function verifyContract(contract, options) {
  const { role, model, thinking, cwd, agentPath, extensions, required } = options;
  const tools = [...roleTools[role], 'structured_output'];
  const launch = contract?.tools;
  if (contract?.version !== 3 || contract.protocol?.packageVersion !== '0.89.0' || contract.context !== 'fresh' ||
      contract.roots?.cwd !== cwd || contract.roots?.artifactPaths || contract.roots?.outputPath || contract.roots?.sessionFile ||
      contract.agent?.name !== `managed-${role}` || contract.agent?.filePath !== agentPath || contract.agent?.source !== 'user' ||
      !contract.launchContractDigest || modelName(contract.model) !== model || (thinking && contract.thinking !== thinking) ||
      contract.systemPromptMode !== 'replace' || contract.inheritProjectContext || contract.inheritGlobalContext || contract.inheritSkills ||
      contract.skills?.requested?.length || contract.intercomBridge?.active !== false) throw new Error('Preflight role, model, or context was not accepted');
  if (!launch?.disableAmbientExtensions || !launch.explicitAllowlist || launch.fanoutAuthorized || launch.effectiveMcpTools?.length ||
      !exact(launch.effectiveAllowlist, tools) || !exact(launch.requiredChildTools, tools) ||
      !exact(launch.configuredExtensions, extensions) || !exact(launch.requiredExtensionIds, required.map(item => item.id)) ||
      !exact(launch.runtimeExtensions, [runtimeExtension]) || !exact(launch.extensionArgs, [runtimeExtension, ...extensions])) {
    throw new Error('Preflight child extensions or tools were not accepted');
  }
  if (contract.diagnostics?.some(item => item.severity !== 'warning' && item.code !== 'host_required')) throw new Error('Preflight reported an unresolved error');
}

function createLease({ stage, model, cwd, state, broker, signal, plan, research, start, priorChecks }) {
  const writes = new Set();
  const checkReceipts = new Map();
  let observed = start;
  let active = false;
  let closed = false;
  let drained = Promise.resolve();
  async function invoke(tool, params, toolSignal) {
    const leaseSignal = toolSignal ? AbortSignal.any([signal, toolSignal]) : signal;
    leaseSignal.throwIfAborted();
    if (closed || !lease.childBound || !roleTools[stage].includes(tool) || active) throw new Error('Role denied or busy');
    active = true;
    const settle = Promise.withResolvers();
    drained = settle.promise;
    try {
      if (tool === 'workflow_write') {
        if (!lease.canWrite(params.path)) throw new Error('Path is not in the approved scope');
        if (observed !== await broker.fingerprint(leaseSignal)) throw new Error('Fingerprint changed before write');
        const result = await broker.write(params, leaseSignal);
        writes.add(params.path);
        observed = await broker.fingerprint(leaseSignal);
        return result;
      }
      if (tool === 'workflow_check') {
        if (!plan?.checks.includes(params.id) || checkReceipts.has(params.id)) throw new Error('Check is not approved or already attempted');
        const current = await broker.fingerprint(leaseSignal);
        const prior = priorChecks?.find(item => item.id === params.id && item.fingerprint === current);
        const result = prior ?? await broker.check(params.id, leaseSignal);
        checkReceipts.set(params.id, result);
        return result;
      }
      if (tool === 'workflow_list') {
        const listed = await broker.list(leaseSignal);
        return listed.slice(0, 512);
      }
      if (tool === 'workflow_read') return await broker.read(params.path, leaseSignal);
      if (tool === 'workflow_search') return await broker.search(params.query, leaseSignal);
      if (tool === 'workflow_diff') return await broker.diff(leaseSignal);
      if (tool === 'workflow_research' && research) {
        return await research(params.topic, leaseSignal);
      }
      throw new Error('Research is unavailable for this launch');
    } finally {
      active = false;
      settle.resolve();
    }
  }
  const lease = { role: stage, workspace: cwd, model, invoke,
    canWrite: path => !closed && !signal.aborted && state.canWrite(path, stage),
    close: async () => { closed = true; await drained; },
    receipts: () => ({ writes: { paths: [...writes], fingerprint: observed }, checks: [...checkReceipts.values()] }),
  };
  return lease;
}

export function createDispatcher(api, emit = () => {}, { timeoutMs = 180000, evidenceReasoning = false } = {}) {
  return async request => {
    const { stage, signal, prompt, plan, copy, models } = request;
    let model = models.core;
    if (stage === 'advisor') model = models.advisor;
    else if (evidenceRoles.has(stage)) model = models.evidence;
    if (!Object.hasOwn(roleTools, stage) || !model) throw new Error('Role or model is not available');
    const cwd = copy.working;
    let thinking;
    if (stage === 'advisor') thinking = 'high';
    else if (evidenceReasoning && evidenceRoles.has(stage)) {
      thinking = stage === 'build-evidence' || request.harderEvidence ? 'medium' : 'low';
    }
    const research = evidenceRoles.has(stage) ? api.research?.() : undefined;
    const task = JSON.stringify({ task: prompt, advisorReason: request.advisorReason, priorPlan: request.refinement, approvedPlan: plan, priorChecks: request.priorChecks, handoffs: request.handoffs, publicResearchActions: research?.ids, repair: request.repair === true });
    const spec = { agent: `managed-${stage}`, cwd, task, context: 'fresh', model, thinking, skill: false,
      timeoutMs, toolBudget: { hard: 32, block: '*' }, artifacts: false, intercomBridge: { mode: 'off' }, result: { kind: 'structured', schema: schemaFor(stage) } };
    const preflight = await api.preflight(spec).catch(() => { throw new Error(`Preflight failed for ${stage}`); });
    signal.throwIfAborted();
    if (!preflight.ok) throw new Error(`Preflight refused ${stage}: ${preflight.code}`);
    verifyContract(preflight.contract, { role: stage, model, thinking, cwd, agentPath: api.agentPath(stage),
      extensions: api.configured(stage), required: api.required });
    let start;
    if (writers.has(stage)) {
      start = await copy.broker.fingerprint(signal);
      if (start !== request.expectedFingerprint) throw new Error('Approved inputs changed before writer launch');
    }
    const lease = createLease({ ...request, cwd, model, broker: copy.broker, start, research });
    const release = bindRole(lease);
    try {
      const result = await api.foreground({ ...spec, requestId: randomUUID(), ownerRunId: api.sessionId, nodeId: stage }, signal);
      signal.throwIfAborted();
      const ack = lease.childBound;
      const usage = Object.fromEntries(['input', 'output', 'cacheRead', 'cacheWrite', 'cost', 'turns', 'toolCalls', 'durationMs']
        .filter(key => Number.isFinite(result.usage?.[key]) && result.usage[key] >= 0).map(key => [key, result.usage[key]]));
      if (Object.keys(usage).length) emit({ type: 'role-usage', role: stage, model, usage });
      if (result.status !== 'completed' || result.agent !== spec.agent || modelName(result.model) !== model ||
          (thinking && result.thinking !== thinking) || preflight.contract.launchContractDigest !== result.launchContractDigest ||
          !ack || ack.role !== stage || ack.model !== model || ack.workspace !== cwd || !exact(ack.tools, [...roleTools[stage], 'structured_output']) ||
          result.result?.kind !== 'structured') throw new Error(`Foreground ${stage} did not produce verified launch evidence (${result.status})`);
      const handoff = result.result.value;
      if (!validHandoff(alias(stage), handoff, plan)) throw new Error(`Invalid ${stage} handoff`);
      return { handoff, ...lease.receipts() };
    } finally {
      try { await lease.close(); } finally { release(); }
    }
  };
}
