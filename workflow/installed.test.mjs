import test from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { here } from './host-policy.mjs';

const run = promisify(execFile);

test('the installed foreground bridge runs fake models with the exact managed tools', { timeout: 30000 }, async t => {
  const root = await mkdtemp(join(tmpdir(), 'managed-native-'));
  t.after(() => rm(root, { force: true, recursive: true }));
  const [home, agent, cwd, source] = ['home', 'agent', 'parent', 'source'].map(name => join(root, name));
  await Promise.all([home, agent, cwd, source].map(path => mkdir(path, { mode: 0o700 })));
  await run(process.execPath, [here('native-fixture.mjs')], { cwd, timeout: 25000,
    env: { PATH: '/usr/bin', HOME: home, TMPDIR: root, PI_CODING_AGENT_DIR: agent, FIXTURE_SOURCE: source,
      XDG_CONFIG_HOME: home, XDG_DATA_HOME: home, XDG_CACHE_HOME: home, JITI_FS_CACHE: 'false' } });
});

test('installed versions, isolated extension factories, and preflight resolve offline with fake models', async t => {
  const root = await mkdtemp(join(tmpdir(), 'managed-installed-'));
  t.after(() => rm(root, { force: true, recursive: true }));
  const [home, agent, cwd] = ['home', 'agent', 'parent'].map(name => join(root, name));
  await Promise.all([home, agent, cwd].map(path => mkdir(path, { mode: 0o700 })));
  const code = `
    import assert from 'node:assert/strict';
    import { isolatedConfiguration, verifyVersions } from ${JSON.stringify(here('sdk-host.mjs'))};
    import { loadExtensions } from ${JSON.stringify(here('../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.js'))};
    import { parentExtensions, controllerKey } from ${JSON.stringify(here('host-policy.mjs'))};
    import { createDispatcher } from ${JSON.stringify(here('dispatch.mjs'))};
    const cwd = process.cwd();
    const agentDir = process.env.PI_CODING_AGENT_DIR;
    const slot = { agentDir };
    globalThis[controllerKey] = slot;
    await verifyVersions();
    await isolatedConfiguration(agentDir);
    const data = await loadExtensions([...parentExtensions], cwd);
    assert.deepEqual(data.errors, []);
    assert.equal(data.extensions.length, 3);
    for (const extension of data.extensions) {
      assert.equal(extension.commands.size, 0);
      assert.equal(extension.shortcuts.size, 0);
    }
    const fakeModel = { provider: 'fixture', id: 'core', reasoning: false };
    const ctx = { sessionManager: { getSessionId: () => 'stub-session', getSessionFile: () => undefined }, model: fakeModel,
      modelRegistry: { getAvailable: () => [fakeModel], getAll: () => [fakeModel] } };
    data.runtime.getAllTools = () => [{ name: 'subagent' }];
    data.runtime.setActiveTools = names => assert.deepEqual(names, []);
    const controller = data.extensions.find(ext => ext.path.endsWith('controller-extension.ts'));
    await controller.handlers.get('session_start')[0]({}, ctx);
    const result = await slot.api.preflight({ agent: 'managed-planner', cwd, context: 'fresh', model: 'fixture/core', task: 'fixture',
      artifacts: false, skill: false, intercomBridge: { mode: 'off' }, result: { schema: { type: 'object' } } });
    assert.equal(result.ok, true, JSON.stringify(result));
    const contract = result.contract;
    assert.equal(contract.agent.filePath, slot.api.agentPath('planner'));
    assert.equal(contract.model, 'fixture/core');
    assert.deepEqual(contract.tools.requiredExtensionIds, ['workflow-permission', 'workflow-guard']);
    assert.deepEqual(contract.tools.effectiveAllowlist, ['workflow_read', 'structured_output']);
    assert.equal(contract.tools.fanoutAuthorized, false);
    assert.equal(contract.tools.disableAmbientExtensions, true);
    const api = slot.api;
    const preflight = api.preflight;
    let checked;
    api.preflight = async spec => { checked = await preflight(spec); return checked; };
    api.foreground = async spec => {
      const child = await loadExtensions([...api.configured('planner'), ...api.required.map(item => item.path)], cwd);
      assert.deepEqual(child.errors, []);
      const guard = child.extensions.find(ext => ext.path.endsWith('child-guard.ts'));
      const tools = child.extensions.find(ext => ext.path.endsWith('child-tools.ts'));
      assert.deepEqual([...tools.tools.keys()], ['workflow_read']);
      let active = ['workflow_read', 'structured_output', 'bash'];
      child.runtime.getActiveTools = () => active;
      child.runtime.getAllTools = () => active.map(name => ({ name })).concat(['workflow_read', 'structured_output'].map(name => ({ name })));
      child.runtime.setActiveTools = names => { active = names; };
      await guard.handlers.get('session_start')[0]({}, { cwd, model: fakeModel });
      assert.deepEqual(active, ['workflow_read', 'structured_output']);
      return { status: 'completed', agent: spec.agent, model: spec.model, launchContractDigest: checked.contract.launchContractDigest,
        result: { kind: 'structured', value: { summary: 'fixture plan', paths: [], checks: [], criteria: ['Use the fixture'] } } };
    };
    const answer = await createDispatcher(api)({ stage: 'planner', signal: new AbortController().signal,
      prompt: 'fixture', copy: { working: cwd, broker: {} }, models: { core: 'fixture/core' } });
    assert.equal(answer.handoff.summary, 'fixture plan');
    await controller.handlers.get('session_shutdown')[0]({}, ctx);
    console.log('offline preflight passed');
  `;
  const { stdout } = await run(process.execPath, ['--input-type=module', '-e', code], { cwd,
    timeout: 25000, env: { PATH: '/usr/bin', HOME: home, TMPDIR: root, PI_CODING_AGENT_DIR: agent,
      XDG_CONFIG_HOME: home, XDG_DATA_HOME: home, XDG_CACHE_HOME: home, JITI_FS_CACHE: 'false' } });
  assert.match(stdout, /offline preflight passed/);
});
