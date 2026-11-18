import { fileURLToPath } from 'node:url';

export const controllerKey = Symbol.for('pi-workflow.sdk-controller.v1');
export const here = name => fileURLToPath(new URL(name, import.meta.url));
export const required = Object.freeze([
  Object.freeze({ id: 'workflow-permission', path: here('permissions-adapter.ts') }),
  Object.freeze({ id: 'workflow-guard', path: here('child-guard.ts') }),
]);
export const subagentConfig = JSON.stringify({
  asyncByDefault: false, forceTopLevelAsync: false, autoResume: false, enableSlashCommands: false,
  globalConcurrencyLimit: 1, parallel: { concurrency: 1, maxTasks: 1 }, maxSubagentSpawnsPerSession: 48,
  fleetView: false, asyncWidget: false, programStatus: false, worktree: false,
  activeRunsPrompt: false, scheduledRuns: { enabled: false }, waitTool: { enabled: false },
  missions: { enabled: false }, intercomBridge: { mode: 'off' }, proactiveSkillSubagents: false,
  disabledFeatures: ['agent-management', 'watchdog', 'panes', 'missions', 'lane-management', 'spawn-budget-grants',
    'gates', 'usage-budgets', 'control-overrides', 'external-machines', 'workflow-scripts'],
});
export const subagentConfigPath = 'extensions/subagent/config.json';
export const configured = () => [here('child-tools.ts')];
export const parentExtensions = Object.freeze(['permissions-adapter.ts', 'subagents-adapter.ts', 'controller-extension.ts'].map(here));
