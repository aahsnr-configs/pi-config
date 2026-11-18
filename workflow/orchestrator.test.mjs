import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkflowState } from './state.mjs';
import { advance } from './orchestrator.mjs';

function fixture(t) {
  const state = new WorkflowState();
  const root = '/fixture';
  state.start('edit a file', root, 'fake/core');
  state.promote(root);
  let held = false;
  let fingerprint = 'first';
  const calls = [];
  const plan = { summary: 'Change a', paths: ['a.txt'], checks: ['node-tests'], criteria: ['Update a'] };
  const answers = {
    scout: { handoff: { summary: 'Found a', paths: ['a.txt'] } },
    planner: { handoff: plan },
    worker: () => {
      assert.equal(state.canWrite('a.txt', 'worker'), true);
      assert.equal(state.canWrite('b.txt', 'worker'), false);
      fingerprint += '+edit';
      return { handoff: { summary: 'Edited a', paths: ['a.txt'] }, writes: { fingerprint, paths: ['a.txt'] } };
    },
    reviewer: { handoff: { verdict: 'accept', summary: 'Looks good' } },
    verifier: () => ({ handoff: { summary: 'Checked' }, checks: [{ id: 'node-tests', status: 'passed', fingerprint }] }),
  };
  const host = {
    tryLock: async () => {
      if (held) return null;
      held = true;
      return () => { held = false; };
    },
    fingerprint: async cwd => {
      assert.equal(cwd, root);
      return fingerprint;
    },
    dispatch: async request => {
      assert.equal(held, true);
      calls.push(request.stage);
      const answer = answers[request.stage];
      return typeof answer === 'function' ? answer(request) : answer;
    },
  };
  t.after(() => assert.equal(held, false));
  return { state, host, calls, answers, run: () => advance(state, host), held: () => held,
    approve: () => state.approve(root, fingerprint), drift: () => { fingerprint += '+drift'; } };
}

test('advancing reuses valid receipts and binds checks to the new worker state', async t => {
  const f = fixture(t);
  assert.equal((await f.run()).status, 'approval-required');
  assert.equal((await f.run()).status, 'approval-required');
  assert.throws(() => f.state.approve('/wrong', 'first'));
  f.approve();
  assert.deepEqual(await f.run(), { status: 'completed', verification: {
    status: 'passed', checks: [{ id: 'node-tests', status: 'passed', fingerprint: 'first+edit' }],
  } });
  assert.deepEqual(f.calls, ['scout', 'planner', 'worker', 'reviewer', 'verifier']);
  assert.equal(f.state.status.mode, 'plan');
  assert.equal(f.state.canWrite('a.txt', 'worker'), false);
});

test('Verifier reuses only passing host receipts for the exact candidate', async t => {
  const f = fixture(t);
  await f.run();
  f.approve();
  f.state.task.checkEvidence = { checks: [
    { id: 'node-tests', status: 'passed', fingerprint: 'first+edit' },
    { id: 'python-tests', status: 'passed', fingerprint: 'stale' },
  ] };
  f.answers.verifier = ({ priorChecks }) => {
    assert.deepEqual(priorChecks.map(item => item.id), ['node-tests']);
    return { handoff: { summary: 'Prior receipt matches' }, checks: [] };
  };
  const result = await f.run();
  assert.equal(result.verification.status, 'passed');
  assert.equal(result.verification.checks[0].fingerprint, 'first+edit');
});

test('drift after approval stops dispatch; approving stale data only reopens planning', async t => {
  const f = fixture(t);
  await f.run();
  f.approve();
  f.drift();
  assert.equal((await f.run()).status, 'stale');
  assert.equal((await f.run()).status, 'stale');
  assert.deepEqual(f.calls, ['scout', 'planner']);
  assert.throws(() => f.state.approve('/fixture', 'first'));
  f.approve();
  assert.equal((await f.run()).status, 'approval-required');
  assert.deepEqual(f.calls, ['scout', 'planner', 'scout', 'planner']);
});

test('one requested repair uses the same scope and cannot run twice', async t => {
  const f = fixture(t);
  f.answers.reviewer = { handoff: { verdict: 'changes-required', summary: 'Repair a' } };
  await f.run();
  f.approve();
  assert.equal((await f.run()).status, 'changes-required');
  assert.equal((await f.run()).status, 'changes-required');
  f.approve();
  assert.equal((await f.run()).status, 'repair-exhausted');
  assert.throws(f.approve);
  assert.deepEqual(f.calls, ['scout', 'planner', 'worker', 'reviewer', 'worker', 'reviewer']);
});

test('bad handoffs or stale check receipts cannot advance', async t => {
  const f = fixture(t);
  const planner = f.answers.planner;
  f.answers.planner = { handoff: { summary: 'Bad scope', paths: ['.env'], checks: [], criteria: ['Never allowed'] } };
  assert.deepEqual(await f.run(), { status: 'failed', stage: 'planner' });
  assert.equal(f.state.task.receipts.has('planner'), false);
  f.answers.planner = planner;
  await f.run();
  f.approve();
  f.answers.verifier = { handoff: { summary: 'Looks OK' }, checks: [{ id: 'node-tests', status: 'passed', fingerprint: 'first' }] };
  assert.deepEqual(await f.run(), { status: 'failed', stage: 'verifier' });
  f.answers.verifier.checks = [];
  assert.equal((await f.run()).verification.status, 'not-tested');
  assert.equal(f.calls.filter(role => role === 'worker').length, 1);
});

test('exclusive operation does not queue and cancellation holds the lock until dispatch drains', async t => {
  const f = fixture(t);
  const started = Promise.withResolvers();
  const drained = Promise.withResolvers();
  f.answers.scout = async ({ signal }) => {
    started.resolve(signal);
    await drained.promise;
  };
  const active = f.run();
  try {
    const signal = await started.promise;
    await assert.rejects(f.run(), /operation is active/);
    const other = new WorkflowState();
    other.start('concurrent task', '/fixture', 'fake/core');
    other.promote('/fixture');
    assert.deepEqual(await advance(other, f.host), { status: 'busy' });
    assert.equal(other.status.running, false);
    f.state.cancel();
    assert.equal(signal.aborted, true);
    assert.equal(f.state.status.cancelling, true);
    assert.equal(f.held(), true);
  } finally {
    drained.resolve();
  }
  assert.deepEqual(await active, { status: 'cancelled' });
  assert.equal(f.state.task.receipts.size, 0);
  assert.equal(f.state.status.running, false);
});

test('dispatch exception releases the lock and does not record an incomplete handoff', async t => {
  const f = fixture(t);
  f.answers.planner = () => { throw new Error('fake launch failure'); };
  await assert.rejects(f.run(), /fake launch failure/);
  assert.equal(f.state.status.running, false);
  assert.equal(f.held(), false);
  assert.deepEqual([...f.state.task.receipts.keys()], ['scout']);
});
