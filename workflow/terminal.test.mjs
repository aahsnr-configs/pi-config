import test from 'node:test';
import assert from 'node:assert/strict';
import { parseLine, terminal } from './terminal.mjs';

function fixture(t) {
  const events = [];
  const calls = [];
  let live = 'live';
  let staged = 'baseline';
  const plan = { summary: 'Edit a', paths: ['a.txt'], checks: ['node-tests'], criteria: ['Update a'] };
  const behavior = {
    'plan-evidence': () => ({ handoff: { summary: 'Read', paths: ['a.txt'] } }),
    'plan-core': () => ({ handoff: plan }),
    'build-evidence': () => ({ handoff: { summary: 'Reviewed the delta', verdict: 'accept' } }),
    advisor: () => ({ handoff: { summary: 'Reviewed', verdict: 'accept' } }),
    'build-core': ({ state }) => {
      assert.equal(state.canWrite('a.txt', 'build-core'), true);
      assert.equal(state.canWrite('b.txt', 'build-core'), false);
      staged = 'edited';
      return { handoff: { summary: 'Edited', paths: ['a.txt'] }, writes: { paths: ['a.txt'], fingerprint: `${live}:${staged}` } };
    },
  };
  const app = terminal({ source: '/fixture', models: { core: 'fake/core', evidence: 'fake/cheap', advisor: 'fake/strong' },
    createWorkspace: async () => {
      staged = 'baseline';
      return { cleanup: async () => {}, broker: {
        fingerprint: async () => `${live}:${staged}`, assertSource: async () => {},
        diff: async () => ({ paths: ['a.txt'], patch: 'fixture patch' }),
        check: async id => ({ id, status: 'not-tested', fingerprint: `${live}:${staged}` }),
      } };
    }, dispatch: async req => {
      calls.push(req);
      return behavior[req.stage](req);
    }, emit: value => { events.push(value); } });
  t.after(() => app.close());
  return { ...app, events, calls, behavior, drift: () => { live += '-drifted'; }, last: () => events.at(-1) };
}

test('managed terminal owns commands, never handing shell or session operations to Pi', t => {
  const f = fixture(t);
  for (const input of ['/reload', '/trust', '/login p', '/model p/x', '/resume', '/new', '/fork', '/compact', '/perm', '/subagent', '/build extra', '!ls', '!!ls', '@file', '/orchestrator']) {
    assert.equal(parseLine(input).kind, 'denied', input);
    f.submit(input);
    assert.equal(f.last().type, 'denied');
  }
  assert.deepEqual(f.calls, []);
});

test('ordinary prompts and refinements stay read-only; Build performs the one approved pass', async t => {
  const f = fixture(t);
  await f.submit('edit a');
  await f.submit('/plan refine a');
  assert.equal(f.calls[2].refinement.summary, 'Edit a');
  assert.deepEqual(f.calls.map(call => call.stage), ['plan-evidence', 'plan-core', 'plan-evidence', 'plan-core']);
  f.submit('/advisor conflict review conflicting evidence');
  await f.submit('/build');
  assert.deepEqual(f.calls.slice(4).map(call => call.stage), ['build-core', 'build-evidence', 'advisor']);
  assert.equal(f.last().verification.status, 'not-tested');
  f.submit('/status');
  assert.equal(f.last().mode, 'plan');
  await f.submit('/patch');
  assert.equal(f.last().patch, 'fixture patch');
  f.drift();
  await f.submit('/patch');
  assert.equal(f.last().type, 'error');
});

test('Build does not dispatch on live drift, and a busy terminal never queues', async t => {
  const f = fixture(t);
  await f.submit('edit a');
  f.drift();
  await f.submit('/build');
  assert.equal(f.last().type, 'stale');
  assert.equal(f.calls.length, 2);
  const inFlight = f.submit('/plan retry');
  f.submit('/build');
  assert.equal(f.last().type, 'busy');
  f.submit('/cancel');
  await inFlight;
  assert.equal(f.last().message, 'Cancelled.');
  assert.equal(f.calls.length, 2);
});
