const protectedNames = /^(?:\.|(?:auth|oauth|mcp-auth|mcp|models(?:-store)?)\.json(?:\.|$)|(?:package(?:-lock)?\.json|pnpm-lock\.yaml|yarn\.lock|bun\.lockb?|Cargo\.(?:toml|lock)|pyproject\.toml|uv\.lock|requirements(?:-[a-z0-9_-]+)?\.txt|go\.(?:mod|sum)|AGENTS\.md|permission-mode\.json|policy\.json)$|settings\.json(?:\.|$)|sandbox\.json$|keybindings\.json$|sessions$|permission-mode$|pi-subagents$|extensions$|workflow$|agents$|bin$|install$|npm$|node_modules$|mcp\.log|id_rsa|id_ed25519|known_hosts)|(?:credential|secret|token)|\.(?:key|p12|pem|pfx|crt)$/i;

export function permitted(path) {
  return typeof path === 'string' && path.length <= 512 &&
    path.split('/').every(part => /^[a-zA-Z0-9_][a-zA-Z0-9_. @+-]*(?![\s\S])/.test(part) && !protectedNames.test(part));
}

export const editable = path => permitted(path) && !/[ @]/.test(path);

export const stages = Object.freeze(['scout', 'planner', 'worker', 'reviewer', 'verifier']);

export function nextStage(receipts) {
  return stages.find(stage => !receipts.has(stage));
}

const name = model => `${model.provider}/${model.id}`;
function price(model) {
  const { input, output } = model.cost ?? {};
  return [input, output].every(value => Number.isFinite(value) && value >= 0) ? input + output : NaN;
}

export function chooseModels(core, available, pool, advisor) {
  if (!core) {
    throw new Error('Select a core model at launch.');
  }
  if (!available.some(model => name(model) === name(core))) {
    throw new Error('The launch model is not available.');
  }
  const candidates = available.filter(model =>
    name(model) !== name(core) && pool.includes(name(model)) && Number.isFinite(price(model))
  ).sort((a, b) => price(a) - price(b) || name(a).localeCompare(name(b)));
  if (!candidates.length) {
    throw new Error('Configure an available distinct ingestion model.');
  }
  const models = { core: name(core), evidence: name(candidates[0]) };
  if (!advisor) {
    return Object.freeze(models);
  }
  const stronger = candidates.find(model => name(model) === advisor && name(model) !== models.evidence &&
    model.reasoning && price(model) > Math.max(price(core), price(candidates[0])));
  if (!stronger) {
    throw new Error('The approved advisor is unavailable or ineligible.');
  }
  return Object.freeze({ ...models, advisor: name(stronger) });
}
