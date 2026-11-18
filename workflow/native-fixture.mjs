// Invoked only by installed.test.mjs in a disposable, credential-free home.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { ModelRuntime } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';
import { createAssistantMessageEventStream, getCurrentTools } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-ai/dist/index.js';
import { isolatedConfiguration, withManagedTerminal } from './sdk-host.mjs';
import { roleTools } from './roles.mjs';

const cwd = process.cwd();
const agentDir = process.env.PI_CODING_AGENT_DIR;
const source = process.env.FIXTURE_SOURCE;
const scratch = process.env.TMPDIR;
const file = join(source, 'message.txt');
await writeFile(file, 'before\n');
await writeFile(join(source, 'fixture.test.cjs'), `require('node:test')('candidate', () => {
  const fs = require('node:fs');
  require('node:assert/strict').equal(fs.readFileSync('message.txt', 'utf8'), 'after\\n');
});\n`);
await isolatedConfiguration(agentDir);
const modelRuntime = await ModelRuntime.create({ authPath: join(agentDir, 'auth.json'), modelsPath: null,
  modelsStorePath: join(agentDir, 'models-store.json'), refreshOnCreate: false });
const plan = { summary: 'Change one file', paths: ['message.txt'], criteria: ['Message is after'], checks: ['node-tests'] };
const accept = { summary: 'Looks good', verdict: 'accept' };
const work = { summary: 'Edited', paths: plan.paths };
const submit = value => ['structured_output', { value }];
const steps = [];
const calls = [];
let failure;
let paused;
function messageFor(model, content, stopReason, errorMessage) {
  return { role: 'assistant', api: model.api, provider: model.provider, model: model.id, content, errorMessage,
    usage: { input: 2, output: 1, cacheRead: 0, cacheWrite: 0, totalTokens: 3,
      cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } }, stopReason, timestamp: Date.now() };
}
const models = ['core', 'cheap', 'strong'].map((id, i) => ({ id, name: id, input: ['text'], reasoning: id !== 'core',
  contextWindow: 32000, maxTokens: 4000, cost: { input: [2, 1, 8][i], output: [2, 1, 8][i], cacheRead: 0, cacheWrite: 0 } }));
modelRuntime.registerProvider('fixture', {
  api: 'fixture-native', baseUrl: 'https://invalid.example', apiKey: 'fixture-placeholder', models,
  streamSimple(model, context, options) {
    const stream = createAssistantMessageEventStream();
    try {
      const [role, action] = steps.shift() ?? [];
      assert.ok(role, 'Unexpected model call');
      calls.push(role);
      const evidence = ['plan-evidence', 'build-evidence', 'scout'].includes(role);
      const expected = { advisor: 'strong', 'plan-evidence': 'cheap', 'build-evidence': 'cheap', scout: 'cheap' };
      assert.equal(model.id, expected[role] ?? 'core');
      if (role === 'advisor') {
        assert.equal(options.reasoning, 'high');
      }
      if (evidence) {
        assert.equal(options.reasoning, role === 'build-evidence' ? 'medium' : 'low');
      }
      assert.deepEqual(getCurrentTools(context.messages).map(tool => tool.name).sort(), [...roleTools[role], 'structured_output'].sort());
      if (action === 'hold') {
        const abort = () => setTimeout(() => stream.push({ type: 'error', reason: 'aborted', error: messageFor(model, [], 'aborted') }), 100);
        options.signal.addEventListener('abort', abort, { once: true });
        paused?.resolve();
        return stream;
      }
      const toolResult = context.messages.findLast(message => message.role === 'toolResult');
      const [name, args] = typeof action === 'function' ? action(toolResult) : action;
      const message = messageFor(model, [{ type: 'toolCall', id: `fixture_${calls.length}`, name, arguments: args }], 'toolUse');
      stream.push({ type: 'start', partial: { ...message, content: [] } });
      stream.push({ type: 'done', reason: 'toolUse', message });
    } catch (error) {
      failure = error;
      stream.push({ type: 'error', reason: 'error', error: messageFor(model, [], 'error', 'Fixture assertion failed') });
    }
    return stream;
  },
});
const events = [];
await withManagedTerminal({ cwd, agentDir, source, scratch, core: 'fixture/core', ingestion: ['fixture/cheap', 'fixture/strong'],
  advisor: 'fixture/strong', timeoutMs: 6000 }, modelRuntime, async app => {
  const last = () => events.at(-1);
  async function command(line, type) {
    await app.submit(line);
    if (failure) {
      throw failure;
    }
    assert.equal(last().type, type, JSON.stringify(last()));
    return last();
  }
  for (const line of ['!touch canary', '/new', '/reload', '/subagent', '/perm']) {
    await command(line, 'denied');
  }
  await command('/plan', 'plan');
  assert.equal(last().plan, null);

  const config = join(agentDir, 'permission-mode', 'permission-mode.json');
  const baseline = await readFile(config);
  await writeFile(config, '{}');
  await command('/plan Must fail before a provider call', 'error');
  assert.deepEqual(calls, []);
  await writeFile(config, baseline);

  const denied = result => assert.equal(result?.isError, true);
  const scout = { summary: 'Read only', paths: plan.paths };
  steps.push(['plan-evidence', ['bash', { command: 'touch canary' }]], ['plan-evidence', result => {
    denied(result);
    return ['workflow_write', { path: 'canary', text: 'not allowed', expected: null }];
  }], ['plan-evidence', result => { denied(result); return submit(scout); }],
  ['advisor', submit(accept)], ['plan-core', submit(plan)]);
  await command('/advisor decision independently review the change', 'advisor');
  assert.deepEqual((await command('Change message.txt', 'plan')).plan.paths, plan.paths);
  assert.deepEqual(steps, []);

  steps.push(['build-core', ['workflow_write', { path: 'canary', expected: null, text: 'bad' }]],
    ['build-core', result => { denied(result); return ['workflow_read', { path: 'message.txt' }]; }],
    ['build-core', result => {
      let expected;
      try { expected = JSON.parse(result.content[0].text).sha256; }
      catch { assert.fail('Read tool did not return its SHA-256'); }
      return ['workflow_write', { path: 'message.txt', expected, text: 'after\n' }];
    }],
    ['build-core', result => { assert.equal(result.isError, false); return submit(work); }],
    ['build-evidence', ['workflow_diff', {}]], ['build-evidence', result => { assert.match(result.content[0].text, /\+after/); return submit(accept); }]);
  await command('/build', 'build-result');
  const patch = await command('/patch', 'patch');
  assert.deepEqual(patch.paths, plan.paths);
  assert.equal(patch.verification.checks[0].fingerprint, patch.fingerprint);
  assert.equal(await readFile(file, 'utf8'), 'before\n');
  assert.equal((await command('/status', 'status')).mode, 'plan');

  // Keep the approved plan and worker results; only independent roles are missing.
  const beforePromotion = calls.length;
  steps.push(['reviewer', submit(accept)], ['verifier', ['workflow_check', { id: 'node-tests' }]],
    ['verifier', result => { assert.match(result.content[0].text, /fingerprint/); return submit({ summary: 'Host check recorded' }); }]);
  await command('/orchestrator resume', 'build-result');
  assert.deepEqual(calls.slice(beforePromotion), ['reviewer', 'verifier', 'verifier']);

  steps.push(['scout', submit(scout)], ['planner', submit(plan)]);
  assert.equal((await command('/orchestrator Begin a fresh task', 'orchestrator')).status, 'approval-required');
  assert.equal((await command('/orchestrator resume', 'orchestrator')).status, 'approval-required');
  steps.push(['worker', submit({ summary: 'No changes needed', paths: [] })], ['reviewer', submit(accept)],
    ['verifier', submit({ summary: 'Check not attempted' })]);
  await command('/orchestrator approve', 'build-result');
  assert.deepEqual(steps, []);
  await assert.rejects(readFile(join(source, 'canary')), /ENOENT/);

  paused = Promise.withResolvers();
  steps.push(['plan-evidence', 'hold']);
  const active = app.submit('/plan wait for cancellation');
  await paused.promise;
  await command('/cancel', 'cancel');
  await command('/build', 'busy');
  await active;
  assert.equal(last().message, 'Cancelled.');
  assert.equal((await command('/status', 'status')).busy, false);

  steps.push(['plan-evidence', 'hold']);
  assert.match((await command('/plan wait for timeout', 'error')).message, /timed_out/);
  assert.equal((await command('/status', 'status')).busy, false);
  assert.deepEqual(steps, []);
}, event => events.push(event));
