import { editable, permitted } from './policy.mjs';

export const checkIds = Object.freeze(['node-tests', 'python-tests']);
const boundedText = value => typeof value === 'string' && value.trim().length > 0 && value.length <= 8192;
const exactKeys = (value, keys) => value && typeof value === 'object' && !Array.isArray(value) &&
  Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
const uniqueList = (value, max, valid) => Array.isArray(value) && value.length <= max &&
  value.every(valid) && new Set(value).size === value.length;

export function validPlan(value) {
  return exactKeys(value, ['summary', 'paths', 'checks', 'criteria']) && boundedText(value.summary) &&
    value.criteria?.length > 0 && uniqueList(value.criteria, 12, item => boundedText(item) && item.length <= 512) &&
    uniqueList(value.paths, 32, editable) && uniqueList(value.checks, checkIds.length, id => checkIds.includes(id));
}

// Only the model's handoff is validated here. A summary cannot attest that a
// write happened or a check passed; those receipts come from the host tools.
export function validHandoff(stage, value, plan) {
  switch (stage) {
    case 'scout':
      return exactKeys(value, ['summary', 'paths']) && boundedText(value.summary) && uniqueList(value.paths, 32, permitted);
    case 'planner':
      return validPlan(value);
    case 'worker':
      return exactKeys(value, ['summary', 'paths']) && boundedText(value.summary) &&
        uniqueList(value.paths, 32, path => plan?.paths.includes(path));
    case 'reviewer':
      return exactKeys(value, ['summary', 'verdict']) && boundedText(value.summary) &&
        ['accept', 'changes-required'].includes(value.verdict);
    case 'verifier':
      return exactKeys(value, ['summary']) && boundedText(value.summary);
    default:
      return false;
  }
}

export function verification(plan, receipts, fingerprint) {
  if (!Array.isArray(receipts) || receipts.length > plan.checks.length) return null;
  const byId = new Map();
  for (const receipt of receipts) {
    if (!exactKeys(receipt, ['id', 'fingerprint', 'status']) || !plan.checks.includes(receipt.id) ||
        byId.has(receipt.id) || receipt.fingerprint !== fingerprint ||
        !['passed', 'failed', 'not-tested'].includes(receipt.status)) return null;
    byId.set(receipt.id, Object.freeze({ ...receipt }));
  }
  const checks = Object.freeze(plan.checks.map(id => byId.get(id) ?? Object.freeze({ id, fingerprint, status: 'not-tested' })));
  let status = 'passed';
  if (checks.some(check => check.status === 'failed')) status = 'failed';
  else if (!checks.length || checks.some(check => check.status === 'not-tested')) status = 'not-tested';
  return Object.freeze({ status, checks });
}
