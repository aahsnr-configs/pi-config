import test from 'node:test';
import assert from 'node:assert/strict';
import { createDispatcher } from './dispatch.mjs';
import { currentRole, roleTools } from './roles.mjs';
import { callPublicMcp } from './child-mcp.ts';
import { publicResearch } from './research.mjs';
import { here } from './host-policy.mjs';

const runtime = here('../npm/node_modules/pi-subagents/src/runs/shared/subagent-prompt-runtime.js');

const cwd = '/private/working';
function fixture() {
  let sent = 0;
  let fingerprint = 'one';
  const role = 'worker';
  const agent = 'managed-worker';
  const required = [{ path: '/guard', id: 'guard' }];
  const tools = [...roleTools[role], 'structured_output'];
  const options = { sessionId: 'fake-parent', agentPath: () => '/definition', configured: () => ['/tools'], required };
  const contract = { version: 3, context: 'fresh', model: 'fake/core', protocol: { packageVersion: '0.89.0' },
    agent: { name: agent, filePath: '/definition', source: 'user' }, roots: { cwd }, launchContractDigest: 'fake-match',
    intercomBridge: { active: false }, systemPromptMode: 'replace',
    tools: { disableAmbientExtensions: true, explicitAllowlist: true, effectiveAllowlist: tools, requiredChildTools: tools,
      configuredExtensions: ['/tools'], requiredExtensionIds: ['guard'], runtimeExtensions: [runtime], extensionArgs: [runtime, '/tools'] } };
  options.preflight = async () => ({ ok: true, contract });
  options.foreground = async spec => {
    sent += 1;
    assert.equal(spec.context, 'fresh');
    assert.equal(spec.artifacts, false);
    const lease = currentRole();
    lease.childBound = { role, model: 'fake/core', workspace: cwd, tools };
    await assert.rejects(lease.invoke('workflow_write', { path: 'b.txt' }), /not in the approved scope/);
    await assert.rejects(lease.invoke('workflow_check', { id: 'node-tests' }), /denied/);
    await lease.invoke('workflow_write', { path: 'a.txt' });
    return { agent, model: 'fake/core', launchContractDigest: 'fake-match', status: 'completed',
      result: { kind: 'structured', value: { summary: 'Updated file', paths: ['a.txt'] } } };
  };
  const request = { stage: role, prompt: 'make edit', expectedFingerprint: 'one', signal: new AbortController().signal,
    models: { core: 'fake/core', evidence: 'fake/cheap' }, plan: { paths: ['a.txt'], checks: [] },
    state: { canWrite: path => path === 'a.txt' }, copy: { working: cwd,
      broker: { fingerprint: async () => fingerprint, write: async () => { fingerprint = 'two'; } } } };
  return { contract, options, request, count: () => sent };
}

test('dispatcher uses foreground receipt and a guarded child acknowledgement', async () => {
  const f = fixture();
  const result = await createDispatcher(f.options)(f.request);
  assert.deepEqual(result.writes, { paths: ['a.txt'], fingerprint: 'two' });
  assert.equal(f.count(), 1);
  assert.equal(currentRole(), undefined);
});

test('a writer never starts if the approved fingerprint changes during preflight', async () => {
  const f = fixture();
  f.request.expectedFingerprint = 'stale';
  await assert.rejects(createDispatcher(f.options)(f.request), /Approved inputs changed/);
  assert.equal(f.count(), 0);
});

test('no dispatch on unexpected preflight tools, and no success without a guard acknowledgement', async () => {
  const f = fixture();
  f.contract.tools.fanoutAuthorized = true;
  await assert.rejects(createDispatcher(f.options)(f.request), /Preflight child extensions or tools/);
  assert.equal(f.count(), 0);
  f.contract.tools.fanoutAuthorized = false;
  f.options.foreground = async () => ({ status: 'completed', agent: 'managed-worker', model: 'fake/core', launchContractDigest: 'fake-match' });
  await assert.rejects(createDispatcher(f.options)(f.request), /did not produce verified launch evidence/);
  assert.equal(currentRole(), undefined);
});

test('the installed MCP client lazily sends only anonymous catalog calls to three stub endpoints', async () => {
  const calls = [];
  const transport = async (url, options) => {
    assert.equal(options.credentials, 'omit');
    assert.equal(options.redirect, 'error');
    assert.equal(options.headers.has('authorization'), false);
    assert.equal(options.headers.has('cookie'), false);
    const rpc = JSON.parse(options.body);
    const host = new URL(url).hostname;
    if (rpc.method === 'notifications/initialized') return new Response(null, { status: 202 });
    let result;
    if (rpc.method === 'initialize') {
      assert.deepEqual(rpc.params.capabilities, {});
      result = { protocolVersion: rpc.params.protocolVersion, capabilities: { tools: {} }, serverInfo: { name: 'stub', version: '1' } };
    } else {
      assert.equal(rpc.method, 'tools/call');
      calls.push({ host, ...rpc.params });
      result = { content: [{ type: 'text', text: 'x'.repeat(20000) }], isError: false };
    }
    return Response.json({ jsonrpc: '2.0', id: rpc.id, result });
  };
  const inputs = { query: 'Node.js v24 CLI', url: 'https://nodejs.org/docs/latest-v24.x/api/test.html', libraryId: '/nodejs/node/v24.0.0' };
  const research = publicResearch('https://api.keenable.ai/mcp-public', inputs,
    (url, tool, args, signal) => callPublicMcp(url, tool, args, signal, transport));
  const signal = new AbortController().signal;
  await assert.rejects(research('arbitrary private task text', signal), /not allowed/);
  assert.deepEqual(calls, []);
  for (const id of ['exa-search', 'context7-docs', 'keenable-search', 'exa-fetch', 'keenable-fetch']) {
    assert.equal((await research(id, signal)).publicUntrustedEvidence.length, 12000);
  }
  assert.deepEqual(calls.map(({ host, name }) => [host, name]), [
    ['mcp.exa.ai', 'web_search_exa'], ['mcp.context7.com', 'query-docs'],
    ['api.keenable.ai', 'search_web_pages'], ['mcp.exa.ai', 'web_fetch_exa'],
  ]);
  assert.match(calls[1].arguments.libraryId, /v24/);
  assert.deepEqual(calls[3].arguments.urls, [inputs.url]);
  assert.equal((await research('exa-search', signal)).publicUntrustedEvidence.length, 12000);
  assert.equal(calls.length, 4);
  assert.throws(() => publicResearch('https://localhost/mcp', inputs), /allowlist/);
});
