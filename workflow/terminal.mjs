import { verification, validPlan, validHandoff } from './handoffs.mjs';
import { WorkflowState } from './state.mjs';
import { advance } from './orchestrator.mjs';

export function parseLine(line) {
  const input = line.trim();
  if (!input || input.length > 16384 || /\p{Cc}/u.test(input)) return { kind: 'denied' };
  if (input.startsWith('!') || input.startsWith('@')) return { kind: 'denied' };
  if (!input.startsWith('/')) return { kind: 'plan', input };
  const [, name, rest = ''] = /^\/([^ ]+)(?: +(.*))?$/.exec(input) ?? [];
  if (name === 'plan') return { kind: 'plan', input: rest };
  if (name === 'orchestrator' && rest) return { kind: 'orchestrator', input: rest };
  if (name === 'advisor' && /^(decision|conflict|blocker) .{1,500}$/u.test(rest)) return { kind: name, input: rest };
  if (!rest && ['build', 'patch', 'status', 'cancel', 'quit', 'help'].includes(name)) return { kind: name };
  return { kind: 'denied' };
}

export function terminal({ source, models, createWorkspace, dispatch, emit }) {
  const state = new WorkflowState();
  let copy;
  let patch;
  let operation;
  let aborter;
  let advisorApproved = false;

  async function resetCopy(signal) {
    patch = undefined;
    const old = copy;
    copy = undefined;
    if (old) await old.cleanup();
    copy = await createWorkspace(signal);
  }

  const host = {
    // The launcher owns the OS flock for the entire SDK process. The controller
    // additionally rejects reentrant work before awaiting any host operation.
    tryLock: async () => () => {},
    fingerprint: async (_workspace, signal) => copy.broker.fingerprint(signal),
    dispatch: async request => dispatch({ ...request, models, copy, state }),
  };

  async function stage(role, signal, data = {}) {
    state.setStage(role);
    const result = await host.dispatch({ stage: role, signal, prompt: state.task.prompt, plan: state.task.plan, ...data });
    signal.throwIfAborted();
    return result;
  }

  async function plan(input, advisor, caller) {
    if (state.mode !== 'plan') throw new Error('Use /orchestrator resume or /cancel first');
    if (!input) return emit({ type: 'plan', plan: state.task?.plan ?? null });
    const previous = state.task;
    const refines = previous?.completed ? undefined : previous?.plan;
    state.start(refines ? `${previous.prompt}\n\nFollow-up: ${input}` : input, source, models.core);
    const signal = AbortSignal.any([state.begin(), caller]);
    try {
      await resetCopy(signal);
      const before = await copy.broker.fingerprint(signal);
      const evidence = await stage('plan-evidence', signal, { refinement: refines, harderEvidence: Boolean(refines) });
      const advice = advisor ? await stage('advisor', signal, { advisorReason: advisor, handoffs: { evidence: evidence.handoff } }) : undefined;
      const answer = await stage('plan-core', signal, { refinement: refines, handoffs: { evidence: evidence.handoff, advice: advice?.handoff } });
      if (before !== await copy.broker.fingerprint(signal)) throw new Error('The workspace drifted during Plan');
      state.recordPlan(answer.handoff, before);
      state.remember('scout', evidence.handoff, before, before);
      state.remember('planner', answer.handoff, before, before);
      emit({ type: 'plan', plan: state.task.plan, suggestion: answer.handoff.criteria.length >= 4
        ? 'You can use /orchestrator resume for independent review and verification.' : undefined });
    } finally {
      state.end();
    }
  }

  async function preparePatch(signal, checks, expected) {
    signal.throwIfAborted();
    const before = await copy.broker.fingerprint(signal);
    const delta = await copy.broker.diff(signal);
    await copy.broker.assertSource(signal);
    signal.throwIfAborted();
    if (before !== expected || before !== await copy.broker.fingerprint(signal) || delta.paths.some(path => !state.task.plan.paths.includes(path))) {
      throw new Error('Patch scope or fingerprints changed');
    }
    state.task.completed = true;
    const note = checks.status === 'not-tested' ? 'NOT TESTED: at least one required check has no passing receipt.' : undefined;
    patch = { type: 'patch', paths: delta.paths, patch: delta.patch, fingerprint: before, verification: checks, note };
    emit({ type: 'build-result', paths: delta.paths, verification: checks, note, message: 'Use /patch to emit the patch; source unchanged.' });
  }

  async function build(advisor, caller) {
    if (state.mode !== 'plan' || !copy || !validPlan(stripFingerprint(state.task?.plan))) throw new Error('No approved Plan to build');
    patch = undefined;
    const fingerprint = await copy.broker.fingerprint(caller);
    caller.throwIfAborted();
    if (fingerprint !== state.task.plan.fingerprint) {
      return emit({ type: 'stale', message: 'Plan inputs changed. Refine the Plan before approving a new scope.', fingerprint });
    }
    state.approve(source, fingerprint);
    const signal = AbortSignal.any([state.begin(), caller]);
    try {
      const edit = await stage('build-core', signal, { expectedFingerprint: fingerprint });
      if (!validHandoff('worker', edit.handoff, state.task.plan)) throw new Error('Invalid Build handoff');
      const after = await copy.broker.fingerprint(signal);
      const delta = await copy.broker.diff(signal);
      if (edit.writes.fingerprint !== after || !samePaths(edit.writes.paths, delta.paths) || !samePaths(edit.handoff.paths, delta.paths)) {
        throw new Error('Build write evidence and private delta differ');
      }
      state.remember('worker', edit.handoff, fingerprint, after);
      const evidence = await stage('build-evidence', signal, { handoffs: { work: edit.handoff } });
      const advice = advisor ? await stage('advisor', signal, { advisorReason: advisor, handoffs: { evidence: evidence.handoff } }) : undefined;
      if (evidence.handoff.verdict !== 'accept' || (advice && advice.handoff.verdict !== 'accept')) {
        throw new Error('Build review requested changes; refine the Plan');
      }
      const receipts = [];
      for (const id of state.task.plan.checks) receipts.push(await copy.broker.check(id, signal));
      const checks = verification(state.task.plan, receipts, after);
      if (!checks || checks.status === 'failed') return emit({ type: 'checks-failed', verification: checks });
      state.task.checkEvidence = checks;
      await preparePatch(signal, checks, after);
    } finally {
      state.finish();
    }
  }

  async function orchestrate(input, signal) {
    if (input === 'resume' && state.mode !== 'orchestrator') {
      if (!copy) throw new Error('No staged task to resume');
      if (!state.task.receipts.size) await resetCopy(signal);
      state.promote(source);
    } else if (input === 'approve') {
      if (state.mode !== 'orchestrator') throw new Error('No orchestrator approval is pending');
      const current = await copy.broker.fingerprint(signal);
      const pending = state.task.pending;
      if (!pending) throw new Error('No orchestrator approval is pending');
      const stale = pending.status === 'stale';
      if (pending.fingerprint !== current) {
        const result = await advance(state, host, signal);
        return emit({ type: 'orchestrator', ...result, plan: result.status === 'approval-required' ? state.task.plan : undefined });
      }
      state.approve(source, current);
      if (stale) {
        try {
          await resetCopy(signal);
          const next = await copy.broker.fingerprint(signal);
          if (sourceFingerprint(next) !== sourceFingerprint(current)) {
            state.task.pending = Object.freeze({ status: 'stale', stage: 'scout', fingerprint: next });
            return emit({ type: 'orchestrator', ...state.task.pending });
          }
          state.task.checkpoint = next;
        } catch (error) {
          state.finish();
          throw error;
        }
      }
    } else if (input !== 'resume') {
      state.start(input, source, models.core);
      await resetCopy(signal);
      state.promote(source);
    }
    const result = await advance(state, host, signal);
    emit({ type: 'orchestrator', ...result, plan: result.status === 'approval-required' ? state.task.plan : undefined,
      summary: result.status === 'completed' ? state.task.receipts.get('verifier').handoff.summary : result.summary });
    if (result.status === 'completed') await preparePatch(signal, result.verification, state.task.checkpoint);
  }

  function submit(line) {
    const { kind, input } = parseLine(line);
    if (kind === 'status') return emit({ type: 'status', ...state.status, busy: Boolean(operation) });
    if (kind === 'help') return emit({ type: 'help', commands: ['/plan [refinement]', '/build', '/orchestrator <task|resume|approve>', '/patch', '/advisor <decision|conflict|blocker> <reason>', '/status', '/cancel', '/quit'] });
    if (kind === 'cancel' || kind === 'quit') {
      if (operation) { aborter.abort(); state.cancel(); }
      else if (state.mode === 'orchestrator') state.finish();
      return kind === 'quit' ? 'quit' : emit({ type: 'cancel', cancelling: Boolean(operation) });
    }
    if (kind === 'denied') return emit({ type: 'denied', message: 'Not a managed command or prompt.' });
    if (operation) return emit({ type: 'busy', message: 'No input is queued.' });
    if (kind === 'advisor') {
      if (!models.advisor || state.mode !== 'plan') return emit({ type: 'denied', message: 'No advisor available for a Plan or Build pass.' });
      advisorApproved = input;
      return emit({ type: 'advisor', model: models.advisor, nextPassOnly: true });
    }
    const advisor = advisorApproved && (kind === 'build' || (kind === 'plan' && input));
    if (advisor) advisorApproved = false;
    aborter = new AbortController();
    const { signal } = aborter;
    operation = Promise.resolve().then(async () => {
      signal.throwIfAborted();
      if (kind === 'patch') {
        if (!patch || await copy.broker.fingerprint(signal) !== patch.fingerprint) throw new Error('No current patch');
        signal.throwIfAborted();
        emit(patch);
      } else if (kind === 'plan') await plan(input, advisor, signal);
      else if (kind === 'build') await build(advisor, signal);
      else await orchestrate(input, signal);
    }).catch(error => emit({ type: 'error', message: signal.aborted ? 'Cancelled.' : error.message }))
      .finally(() => { operation = undefined; aborter = undefined; });
    return operation;
  }

  return { submit, settle: async () => operation,
    close: async () => { aborter?.abort(); state.cancel(); await operation; await copy?.cleanup(); } };
}

function stripFingerprint(plan) {
  if (!plan) return undefined;
  return { summary: plan.summary, paths: plan.paths, checks: plan.checks, criteria: plan.criteria };
}

const sourceFingerprint = combined => combined.split(':')[0];

function samePaths(left, right) {
  return left.length === right.length && left.every(path => right.includes(path));
}
