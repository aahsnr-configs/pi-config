import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { resolveSubagentLaunchContract } from '../npm/node_modules/pi-subagents/bundle/preflight.js';
import { registerSubagentCapabilityCeiling, resolveCurrentSubagentCapabilityCeiling } from '../npm/node_modules/pi-subagents/bundle/capability-ceiling.js';
import { roleTools } from './roles.mjs';
import { registerRequiredChildExtensions } from '../npm/node_modules/pi-subagents/bundle/required-child-extensions.js';
import { SUBAGENT_DELEGATION_CANCEL_EVENT, SUBAGENT_DELEGATION_REQUEST_EVENT,
  SUBAGENT_DELEGATION_RESPONSE_EVENT } from '../npm/node_modules/pi-subagents/bundle/delegation.js';
import type { ExtensionAPI } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';
import type { SubagentDelegationRequest, SubagentDelegationResponse } from '../npm/node_modules/pi-subagents/src/api/delegation.js';
import type { SubagentLaunchContractInput, SubagentLaunchContractResult } from '../npm/node_modules/pi-subagents/src/api/preflight.js';
import { publicResearch } from './research.mjs';
import { callPublicMcp } from './child-mcp.ts';

import { configured, controllerKey, required, subagentConfig, subagentConfigPath } from './host-policy.mjs';

const resolvePreflight = resolveSubagentLaunchContract as (input: SubagentLaunchContractInput) => Promise<SubagentLaunchContractResult>;
const resolveCeiling = resolveCurrentSubagentCapabilityCeiling as typeof import('../npm/node_modules/pi-subagents/src/api/capability-ceiling.js').resolveCurrentSubagentCapabilityCeiling;

type StructuredRequest = SubagentDelegationRequest & { result: Extract<SubagentDelegationRequest['result'], { kind: 'structured' }> };
type Spec = Omit<StructuredRequest, 'requestId' | 'ownerRunId' | 'nodeId'>;
type Host = { agentDir: string; allowResearch: boolean; keenable: string; researchInputs: Parameters<typeof publicResearch>[1]; api?: {
  sessionId: string; required: typeof required; configured: typeof configured;
  agentPath(role: string): string; research(): ReturnType<typeof publicResearch> | undefined;
  preflight(spec: Spec): Promise<SubagentLaunchContractResult>;
  foreground(request: StructuredRequest, signal: AbortSignal): Promise<SubagentDelegationResponse>;
} };

export default function controller(pi: ExtensionAPI) {
  const host = (globalThis as typeof globalThis & { [controllerKey]?: Host })[controllerKey];
  if (!host || host.api) throw new Error('Managed SDK host slot missing or used');
  let registration: ReturnType<typeof registerRequiredChildExtensions> | undefined;
  let ceiling: ReturnType<typeof registerSubagentCapabilityCeiling> | undefined;
  const context7Key = process.env.CONTEXT7_API_KEY;
  const research = host.allowResearch ? publicResearch(host.keenable, host.researchInputs,
    (url, tool, args, signal) => callPublicMcp(url, tool, args, signal, fetch, context7Key)) : undefined;
  const assertConfig = () => {
    if (readFileSync(join(host.agentDir, subagentConfigPath), 'utf8') !== subagentConfig) {
      throw new Error('Managed subagents configuration changed');
    }
  };
  pi.on('session_start', (_event, ctx) => {
    assertConfig();
    const sessionId = ctx.sessionManager.getSessionId();
    if (!sessionId || ctx.sessionManager.getSessionFile()) {
      throw new Error('In-memory parent session missing');
    }
    pi.setActiveTools([]);
    registration = registerRequiredChildExtensions({ sessionId, extensions: required, requireForAllRunners: true });
    host.api = {
      sessionId, required, configured,
      agentPath: role => join(host.agentDir, 'agents', `managed-${role}.md`),
      research: () => research,
      preflight: spec => {
        assertConfig();
        const role = spec.agent.slice('managed-'.length);
        if (spec.agent !== `managed-${role}` || !Object.hasOwn(roleTools, role)) throw new Error('Unrecognized managed role');
        ceiling?.dispose();
        ceiling = registerSubagentCapabilityCeiling({ sessionId, source: 'managed-workflow',
          ceiling: { allowedAgents: [spec.agent], allowedTools: [...roleTools[role], 'structured_output'] } });
        return resolvePreflight({ ...spec, parentSessionId: sessionId,
          parentSessionFile: null, agentScope: 'user', projectTrusted: false, outputSchema: spec.result.schema,
          capabilityCeiling: resolveCeiling(sessionId),
          parentModel: ctx.model, availableModels: ctx.modelRegistry.getAvailable(),
          scopedModelIds: ctx.scopedModels?.map(({ model }) => `${model.provider}/${model.id}`),
          knownProviders: () => [...new Set(ctx.modelRegistry.getAll().map(model => model.provider))], runtimeSnapshotHost: pi });
      },
      foreground: (request, signal) => new Promise((resolve, reject) => {
        if (signal.aborted) { reject(signal.reason); return; }
        const identity = { requestId: request.requestId, ownerRunId: request.ownerRunId, nodeId: request.nodeId };
        const cancel = () => pi.events.emit(SUBAGENT_DELEGATION_CANCEL_EVENT, identity);
        const unsubscribe = pi.events.on(SUBAGENT_DELEGATION_RESPONSE_EVENT, value => {
          if (!value || typeof value !== 'object') return;
          const response = value as SubagentDelegationResponse;
          if (response.requestId !== identity.requestId || response.ownerRunId !== identity.ownerRunId || response.nodeId !== identity.nodeId) return;
          unsubscribe();
          signal.removeEventListener('abort', cancel);
          resolve(response);
        });
        signal.addEventListener('abort', cancel, { once: true });
        pi.events.emit(SUBAGENT_DELEGATION_REQUEST_EVENT, request);
        if (signal.aborted) cancel();
      }),
    };
  });
  pi.on('session_shutdown', () => { ceiling?.dispose(); registration?.dispose(); host.api = undefined; });
  pi.on('tool_call', () => ({ block: true, reason: 'Parent has no model tool authority' }));
  pi.on('session_before_switch', () => ({ cancel: true }));
  pi.on('session_before_fork', () => ({ cancel: true }));
  pi.on('session_before_tree', () => ({ cancel: true }));
  pi.on('session_before_compact', () => ({ cancel: true }));
  pi.on('user_bash', () => ({ result: { output: 'Shell disabled', exitCode: 1, cancelled: false, truncated: false } }));
}
