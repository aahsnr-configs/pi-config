const discover = ['workflow_list', 'workflow_search', 'workflow_read'];
const inspect = ['workflow_read', 'workflow_diff'];

export const roleTools = Object.freeze(Object.fromEntries(Object.entries({
  scout: [...discover, 'workflow_research'],
  planner: ['workflow_read'],
  worker: [...discover, 'workflow_diff', 'workflow_write'],
  reviewer: inspect,
  verifier: [...inspect, 'workflow_check'],
  'plan-evidence': [...discover, 'workflow_research'],
  'plan-core': ['workflow_read'],
  'build-evidence': ['workflow_diff', 'workflow_research'],
  'build-core': [...discover, 'workflow_diff', 'workflow_write'],
  advisor: inspect,
}).map(([role, tools]) => [role, Object.freeze(tools)])));

const activeKey = Symbol.for('pi-workflow.active-child.v1');

export function bindRole(lease) {
  if (globalThis[activeKey]) throw new Error('A workflow role is already active.');
  globalThis[activeKey] = lease;
  return () => {
    if (globalThis[activeKey] === lease) delete globalThis[activeKey];
  };
}

export function currentRole() {
  return globalThis[activeKey];
}
