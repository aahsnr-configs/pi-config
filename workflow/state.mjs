import { randomUUID } from 'node:crypto';
import { permitted } from './policy.mjs';
import { validHandoff, validPlan } from './handoffs.mjs';

export class WorkflowState {
  mode = 'plan';
  task = null;
  #operation;

  get running() {
    return this.#operation !== undefined;
  }

  get status() {
    return { mode: this.mode, stage: this.#operation?.stage ?? this.task?.pending?.stage,
      running: this.running, cancelling: this.#operation?.controller.signal.aborted ?? false,
      pending: this.task?.pending?.status };
  }

  begin() {
    if (this.running) throw new Error('Another workflow operation is active.');
    this.#operation = { controller: new AbortController() };
    return this.#operation.controller.signal;
  }

  setStage(stage) {
    if (!this.#operation) throw new Error('No workflow operation is active.');
    this.#operation.stage = stage;
  }

  cancel() {
    this.#operation?.controller.abort();
  }

  end() {
    this.#operation = undefined;
  }

  start(prompt, workspace, core) {
    if (this.running || this.mode !== 'plan') throw new Error('Finish or cancel the current workflow.');
    if (typeof prompt !== 'string' || !prompt.trim() || prompt.length > 16384) throw new Error('Enter a bounded task first.');
    this.task = {
      id: randomUUID(), prompt, workspace, core,
      receipts: new Map(), approval: null, plan: null,
      checkpoint: null, pending: null, repairUsed: false, repairing: false,
    };
    return this.task;
  }

  recordPlan(plan, fingerprint) {
    if (!this.task || !['plan', 'orchestrator'].includes(this.mode)) throw new Error('No planning task is active.');
    if (!validPlan(plan) || !fingerprint) throw new Error('Invalid plan scope or fingerprint.');
    this.task.plan = Object.freeze({ ...freezeHandoff(plan), fingerprint });
    this.task.approval = null;
  }

  remember(stage, value, before, after, verification) {
    const task = this.task;
    if (!task || !validHandoff(stage, value, task.plan) || !before || !after) throw new Error('Invalid role receipt.');
    task.receipts.set(stage, Object.freeze({ taskId: task.id, stage, before, after, handoff: freezeHandoff(value), verification }));
    task.checkpoint = after;
  }

  approve(workspace, fingerprint) {
    const task = this.task;
    if (this.running || !task || workspace !== task.workspace || !fingerprint) {
      throw new Error('Approval requires the current idle workflow and workspace.');
    }
    if (this.mode === 'orchestrator') return this.#approveOrchestrator(task, fingerprint);
    if (this.mode !== 'plan' || fingerprint !== task.plan?.fingerprint) throw new Error('Review a current plan before approving writes.');
    this.mode = 'build';
    task.approval = new Set(task.plan.paths);
  }

  #approveOrchestrator(task, fingerprint) {
    const pending = task.pending;
    if (!pending || fingerprint !== pending.fingerprint) throw new Error('Review the current approval request.');
    if (pending.status === 'stale') {
      task.receipts.clear();
      Object.assign(task, { plan: null, approval: null, repairing: false, checkpoint: fingerprint, pending: null });
      return;
    }
    if (pending.status === 'changes-required' && !task.repairUsed) {
      task.repairUsed = true;
      task.repairing = true;
    } else if (pending.status !== 'approval-required' || fingerprint !== task.plan?.fingerprint) {
      throw new Error('No matching write approval is pending.');
    }
    task.pending = null;
    task.approval = new Set(task.plan.paths);
  }

  promote(workspace) {
    if (this.running || this.mode !== 'plan' || !this.task || this.task.workspace !== workspace) {
      throw new Error('No idle planning task is available in this workspace.');
    }
    this.mode = 'orchestrator';
    if (!this.task.receipts.has('planner')) this.task.plan = null;
    this.task.approval = null;
  }

  canWrite(path, stage) {
    const writer = { build: 'build-core', orchestrator: 'worker' }[this.mode];
    return writer !== undefined && stage === writer && this.#operation?.stage === stage &&
      !this.#operation.controller.signal.aborted && permitted(path) && !!this.task?.approval?.has(path);
  }

  finish() {
    this.end();
    this.mode = 'plan';
    if (this.task) this.task.approval = null;
  }
}

function freezeHandoff(value) {
  return Object.freeze(Object.fromEntries(Object.entries(value).map(([key, item]) =>
    [key, Array.isArray(item) ? Object.freeze([...item]) : item])));
}
