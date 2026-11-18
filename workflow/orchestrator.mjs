import { stages } from './policy.mjs';
import { validHandoff, verification } from './handoffs.mjs';

function pause(task, status, stage, fingerprint, details = {}) {
  task.approval = null;
  task.pending = Object.freeze({ status, stage, fingerprint, ...details });
  return task.pending;
}

async function fingerprint(host, workspace, signal) {
  signal.throwIfAborted();
  const value = await host.fingerprint(workspace, signal);
  signal.throwIfAborted();
  if (typeof value !== 'string' || !value) {
    throw new Error('No workspace fingerprint.');
  }
  return value;
}

async function runStage(state, host, signal, stage, before) {
  const task = state.task;
  const answer = await host.dispatch({
    stage, signal, prompt: task.prompt, workspace: task.workspace, core: task.core, plan: task.plan,
    repair: task.repairing, expectedFingerprint: before,
    priorChecks: stage === 'verifier' ? passingChecks(task, before) : undefined,
    handoffs: Object.freeze(Object.fromEntries([...task.receipts].map(([role, receipt]) => [role, receipt.handoff]))),
  });
  const after = await fingerprint(host, task.workspace, signal);
  return assess(state, stage, answer, before, after);
}

function assess(state, stage, answer, before, after) {
  const task = state.task;
  // Only the host-side write broker may produce write evidence. A model's
  // structured output must never be copied onto this part of the envelope.
  const written = stage === 'worker' ? answer?.writes : undefined;
  if ((written?.fingerprint ?? before) !== after) {
    return pause(task, 'stale', stage, after);
  }
  const value = answer?.handoff;
  if (!validHandoff(stage, value, task.plan)) {
    return { status: 'failed', stage };
  }
  if (stage === 'worker' && !matchesWrites(state, value.paths, written, after)) {
    return { status: 'failed', stage };
  }
  let checks;
  if (stage === 'verifier') {
    if (!Array.isArray(answer.checks)) {
      return { status: 'failed', stage };
    }
    const attempted = new Set(answer.checks.map(item => item?.id));
    const retained = passingChecks(task, after).filter(item => !attempted.has(item.id));
    checks = verification(task.plan, [...retained, ...answer.checks], after);
  }
  if (checks === null) {
    return { status: 'failed', stage };
  }
  if (stage === 'planner') {
    state.recordPlan(value, after);
  }
  state.remember(stage, value, before, after, checks);
  return afterStage(task, stage, value, after, checks);
}

function afterStage(task, stage, value, after, checks) {
  if (stage === 'worker' && task.repairing) {
    task.repairing = false;
    task.receipts.delete('reviewer');
    task.receipts.delete('verifier');
  }
  if (stage === 'reviewer' && value.verdict === 'changes-required') {
    const status = task.repairUsed ? 'repair-exhausted' : 'changes-required';
    return pause(task, status, stage, after, { summary: value.summary });
  }
  if (checks?.status === 'failed') {
    return pause(task, 'checks-failed', stage, after, { verification: checks });
  }
  return null;
}

function passingChecks(task, fingerprint) {
  return (task.checkEvidence?.checks ?? []).filter(item => item.status === 'passed' && item.fingerprint === fingerprint);
}

function matchesWrites(state, reported, written, after) {
  const actual = written?.paths;
  return written?.fingerprint === after && Array.isArray(actual) && actual.length === reported.length &&
    new Set(actual).size === actual.length &&
    actual.every(path => state.canWrite(path, 'worker') && reported.includes(path));
}

async function walk(state, host, signal) {
  const task = state.task;
  const current = await fingerprint(host, task.workspace, signal);
  const pending = pendingAt(task, current);
  if (pending) {
    return pending;
  }
  task.checkpoint = current;
  for (const stage of stages) {
    signal.throwIfAborted();
    if (task.receipts.has(stage) && !(stage === 'worker' && task.repairing)) {
      continue;
    }
    state.setStage(stage);
    if (stage === 'worker' && !task.approval) {
      return pause(task, task.repairing ? 'repair-exhausted' : 'approval-required', stage, task.checkpoint);
    }
    const before = await fingerprint(host, task.workspace, signal);
    if (before !== task.checkpoint) {
      return pause(task, 'stale', stage, before);
    }
    const result = await runStage(state, host, signal, stage, before);
    if (stage === 'worker') {
      task.approval = null;
    }
    if (result) {
      return result;
    }
  }
  return { status: 'completed', verification: task.receipts.get('verifier').verification };
}

function pendingAt(task, current) {
  const expected = task.pending?.fingerprint ?? task.checkpoint;
  if (!expected || expected === current) {
    return task.pending;
  }
  const stage = task.pending?.stage ?? stages.find(role => !task.receipts.has(role)) ?? 'verifier';
  return pause(task, 'stale', stage, current);
}

/** Host must acquire a non-queuing per-user lock and settle all descendants
 * before dispatch resolves or rejects, including after signal cancellation. */
export async function advance(state, host, callerSignal) {
  if (state.mode !== 'orchestrator' || !state.task) {
    throw new Error('The orchestrator is unavailable.');
  }
  const ownSignal = state.begin();
  const signal = callerSignal ? AbortSignal.any([ownSignal, callerSignal]) : ownSignal;
  let release;
  let result;
  try {
    release = await host.tryLock();
    if (typeof release !== 'function') {
      return { status: 'busy' };
    }
    signal.throwIfAborted();
    result = await walk(state, host, signal);
    return result;
  } catch (error) {
    if (!signal.aborted) {
      throw error;
    }
    return { status: 'cancelled' };
  } finally {
    try {
      if (typeof release === 'function') {
        state.task.approval = null;
        await release();
      }
    } finally {
      if (result?.status === 'completed') {
        state.finish();
      }
      else {
        state.end();
      }
    }
  }
}
