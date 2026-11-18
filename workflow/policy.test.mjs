import test from 'node:test';
import assert from 'node:assert/strict';
import { permitted, nextStage, chooseModels } from './policy.mjs';

test('protected paths are rejected across roles', () => {
  for (const path of ['../a', '/a', 'a/.env.test', 'auth.json', 'sessions/a', 'a/private.pem', 'workflow/controller.ts', 'settings.json']) {
    assert.equal(permitted(path), false, path);
  }
  assert.equal(permitted('src/main.ts'), true);
});

test('promotion skips stages with validated receipts', () => {
  assert.equal(nextStage(new Set(['scout', 'planner', 'worker'])), 'reviewer');
});

test('core stays fixed and evidence uses the cheapest distinct model', () => {
  const core = { provider: 'p', id: 'core', cost: { input: 2, output: 2 } };
  const cheap = { provider: 'p', id: 'cheap', cost: { input: 1, output: 1 } };
  assert.deepEqual(chooseModels(core, [core, cheap], ['p/core', 'p/cheap']), { core: 'p/core', evidence: 'p/cheap' });
  assert.throws(() => chooseModels(core, [cheap], ['p/cheap']), /launch model is not available/);
  assert.throws(() => chooseModels(core, [core], ['p/core']), /distinct ingestion model/);
  assert.throws(() => chooseModels(core, [core, cheap], ['p/cheap'], 'p/advisor'), /advisor is unavailable/);
  const advisor = { provider: 'p', id: 'advisor', reasoning: true, cost: { input: 8, output: 8 } };
  assert.deepEqual(chooseModels(core, [core, advisor, cheap], ['p/cheap', 'p/advisor'], 'p/advisor'),
    { core: 'p/core', evidence: 'p/cheap', advisor: 'p/advisor' });
});

test('Build writes require current approval and the active Build role', async () => {
  const { WorkflowState } = await import('./state.mjs');
  const state = new WorkflowState();
  state.start('edit a file', '/workspace', 'p/core');
  state.recordPlan({ summary: 'Edit a', paths: ['src/a'], checks: [], criteria: ['Update a'] }, 'baseline');
  assert.throws(() => state.approve('/workspace', 'changed'));
  assert.equal(state.canWrite('src/a', 'build-core'), false);
  state.approve('/workspace', 'baseline');
  state.begin();
  state.setStage('build-evidence');
  assert.equal(state.canWrite('src/a', 'build-core'), false);
  state.setStage('build-core');
  assert.equal(state.canWrite('src/a', 'worker'), false);
  assert.equal(state.canWrite('src/a', 'build-core'), true);
  assert.equal(state.canWrite('src/b', 'build-core'), false);
  state.cancel();
  assert.equal(state.canWrite('src/a', 'build-core'), false);
  state.finish();
});

test('standalone plan is not a delegated Planner receipt', async () => {
  const { WorkflowState } = await import('./state.mjs');
  const state = new WorkflowState();
  state.start('edit a', '/workspace', 'p/core');
  state.recordPlan({ summary: 'Edit a', paths: ['src/a'], checks: [], criteria: ['Update a'] }, 'baseline');
  state.promote('/workspace');
  assert.equal(state.task.receipts.size, 0);
  assert.equal(state.task.plan, null);
  assert.throws(() => state.approve('/workspace', 'baseline'));
});
