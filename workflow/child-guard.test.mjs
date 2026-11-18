import test from 'node:test';
import assert from 'node:assert/strict';
import guard from './child-guard.ts';
import { bindRole, roleTools } from './roles.mjs';

function fixture(t, role = 'worker') {
  const lease = { role, model: 'fake/core', workspace: '/fixture', canWrite: path => path === 'allowed.txt' };
  const release = bindRole(lease);
  t.after(release);
  const handlers = new Map();
  const declared = [...roleTools[role], 'structured_output', 'bash', 'subagent', 'codemode'];
  let active = [...declared];
  guard({ on: (event, handler) => handlers.set(event, handler),
    getActiveTools: () => active, setActiveTools: names => { active = names; },
    getAllTools: () => declared.map(name => ({ name })),
  });
  const ctx = { model: { provider: 'fake', id: 'core' }, cwd: '/fixture' };
  const emit = (event, payload) => handlers.get(event)(payload, ctx);
  const call = (toolName, input = {}) => emit('tool_call', { toolName, input });
  return { call, ctx, emit, lease, release, active: () => active, drop: name => { active = active.filter(tool => tool !== name); } };
}

test('child only acknowledges the pinned role, model, workspace, and exact active tools', t => {
  const f = fixture(t);
  assert.equal(f.call('structured_output').block, true);
  f.emit('session_start');
  assert.deepEqual(f.lease.childBound, { role: 'worker', model: 'fake/core', workspace: '/fixture', tools: f.active() });
  assert.deepEqual(f.active(), [...roleTools.worker, 'structured_output']);
  for (const name of ['bash', 'subagent', 'codemode', 'workflow_check', 'workflow_research']) assert.equal(f.call(name).block, true);
  assert.equal(f.call('workflow_write', { path: 'other.txt' }).block, true);
  assert.equal(f.call('workflow_write', { path: 'allowed.txt' }), undefined);
  assert.equal(f.emit('user_bash').result.exitCode, 1);
  assert.equal(f.emit('session_before_switch').cancel, true);
  assert.equal(f.emit('session_before_fork').cancel, true);
  assert.equal(f.emit('session_before_tree').cancel, true);
  f.ctx.model.id = 'different';
  assert.equal(f.call('workflow_write', { path: 'allowed.txt' }).block, true);
  f.release();
  assert.equal(f.call('structured_output').block, true);
});

test('a denied required tool fails closed even if the host catches startup errors', t => {
  const f = fixture(t, 'planner');
  f.drop('workflow_read');
  assert.throws(() => f.emit('session_start'), /Required workflow tool unavailable/);
  assert.deepEqual(f.active(), []);
  assert.equal(f.lease.childBound, undefined);
  assert.equal(f.call('workflow_read').block, true);
  assert.equal(f.call('structured_output').block, true);
});
