import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { loadExtensions } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.js';
import { createJiti } from '../install/releases/1.5.2/node_modules/jiti/lib/jiti.mjs';

const here = path => fileURLToPath(new URL(path, import.meta.url));
const proposal = 'propose-prompt-optimization';

test('installed Promptsmith accepts and restores the prepared approval settings', async () => {
  const jiti = createJiti(import.meta.url, { fsCache: false, moduleCache: false,
    alias: { '@earendil-works/pi-tui': here('../install/releases/1.5.2/node_modules/@earendil-works/pi-tui/dist/index.js') } });
  const { PromptsmithRuntimeState, sanitizeSettings } = await jiti.import(here('../npm/node_modules/pi-promptsmith/src/state.ts'));
  const path = here('../promptsmith-settings.json');
  const config = JSON.parse(readFileSync(path, 'utf8'));
  assert.deepEqual(sanitizeSettings(config), config);
  const state = new PromptsmithRuntimeState(path);
  state.restoreSettings();
  assert.deepEqual(state.getSettings(), config);
  assert.equal(config.previewBeforeReplace, true);
  assert.equal(config.autoSendEnhancedPrompt, false);
  assert.equal(config.rewriteMode, 'execution-contract');
});

async function fixture(actions) {
  const paths = [here('../npm/node_modules/pi-prompt-optimize/extensions/prompt-optimize.ts'),
    here('../extensions/prompt-review-guard.ts')];
  const loaded = await loadExtensions(paths, here('../'));
  assert.deepEqual(loaded.errors, []);
  const optimizer = loaded.extensions.find(ext => ext.path === paths[0]);
  const guard = loaded.extensions.find(ext => ext.path === paths[1]);
  const state = { entries: [], leaf: 'origin', idle: true, tools: ['read', 'bash', 'write'],
    editor: 'structured original', messages: [], writes: [], waits: [] };
  const ctx = { cwd: here('../'), mode: 'tui', model: { provider: 'fixture', id: 'model' },
    isIdle: () => state.idle,
    sessionManager: { getBranch: () => state.entries, getSessionId: () => 'fixture', getLeafId: () => state.leaf },
    ui: { setStatus() {}, notify() {}, theme: { fg: (_color, text) => text },
      select: async () => actions.shift(), editor: async () => 'Preserve the original paths',
      setEditorText: text => { state.editor = text; state.writes.push(text); } },
    waitForIdle: () => state.idle ? Promise.resolve() : new Promise(resolve => state.waits.push(resolve)),
    navigateTree: async target => {
      assert.equal(target, 'origin'); state.entries = []; state.leaf = target;
      await emit('session_tree'); return { cancelled: false };
    }, abort: () => { state.idle = true; } };
  const emit = async name => {
    for (const ext of loaded.extensions) for (const handler of ext.handlers.get(name) || []) await handler({}, ctx);
  };
  loaded.runtime.getActiveTools = () => state.tools;
  loaded.runtime.setActiveTools = names => { state.tools = names; };
  loaded.runtime.getAllTools = () => [{ name: 'read', sourceInfo: { source: 'builtin', path: 'builtin:read' } },
    ...[...optimizer.tools].map(([name, tool]) => ({ name, sourceInfo: tool.sourceInfo }))];
  loaded.runtime.appendEntry = (customType, data) => {
    state.entries.push({ type: 'custom', customType, data }); state.leaf = `entry-${state.entries.length}`;
  };
  loaded.runtime.sendUserMessage = text => state.messages.push(text);
  const call = (toolName, input = {}) => guard.handlers.get('tool_call')[0]({ toolName, input }, ctx);
  const command = args => optimizer.commands.get('prompt-optimize').handler(args, ctx);
  const propose = prompt => optimizer.tools.get(proposal).definition.execute('fixture-call', { prompt },
    new AbortController().signal, undefined, ctx);
  const settle = async () => {
    state.idle = true;
    for (const resolve of state.waits.splice(0)) resolve();
    await new Promise(resolve => setImmediate(resolve));
    await emit('agent_settled');
  };
  await emit('session_start');
  return { state, emit, call, command, propose, settle };
}

test('installed optimizer Accept returns text without sending a coding turn', async () => {
  const f = await fixture(['Accept']);
  await f.command('structured draft'); f.state.idle = false; await f.emit('before_agent_start');
  assert.equal(f.call('bash').block, true);
  assert.equal(f.call(proposal), undefined);
  const result = await f.propose('final approved prompt');
  assert.equal(result.terminate, true);
  assert.equal(f.call('write').block, true);
  assert.deepEqual(f.state.writes, []);
  await f.settle();
  assert.equal(f.state.editor, 'final approved prompt');
  assert.equal(f.call('write'), undefined);
  assert.equal(f.state.messages.length, 1); // Only the optimizer kickoff, never the accepted task.
  assert.match(f.state.messages[0], /structured draft/);
});

test('installed Modify retains restrictions; Cancel inserts no proposal', async () => {
  const f = await fixture(['Modify', 'Cancel']);
  await f.command('structured draft'); f.state.idle = false;
  const modified = await f.propose('first candidate');
  assert.equal(modified.details.action, 'Modify');
  assert.equal(f.call('bash').block, true);
  assert.deepEqual(f.state.writes, []);
  await f.propose('second candidate'); await f.settle();
  assert.deepEqual(f.state.writes, []);
  assert.equal(f.call('bash'), undefined);
  assert.equal(f.state.messages.length, 1);
});

test('installed explicit end returns to the branch start without inserting or submitting', async () => {
  const f = await fixture([]);
  await f.command('structured draft'); assert.equal(f.call('bash').block, true);
  await f.command('end');
  assert.equal(f.call('bash'), undefined);
  assert.deepEqual(f.state.writes, []);
  assert.equal(f.state.messages.length, 1);
});
