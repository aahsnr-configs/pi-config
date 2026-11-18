import Type, { type TSchema } from '../install/releases/1.5.2/node_modules/typebox/build/index.mjs';
import type { ExtensionAPI } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';
import { currentRole, roleTools } from './roles.mjs';
import { checkIds } from './handoffs.mjs';
import { researchIds } from './research.mjs';

const path = Type.String({ minLength: 1, maxLength: 512 });
const id = (ids: readonly string[]) => Type.Union(ids.map(value => Type.Literal(value)));
const exact = { additionalProperties: false };
const schemas = {
  workflow_list: Type.Object({}, exact),
  workflow_search: Type.Object({ query: Type.String({ minLength: 1, maxLength: 100 }) }, exact),
  workflow_read: Type.Object({ path }, exact),
  workflow_write: Type.Object({ path, expected: Type.Union([Type.Null(), Type.String({ pattern: '^[a-f0-9]{64}$' })]),
    text: Type.String({ maxLength: 262144 }) }, exact),
  workflow_diff: Type.Object({}, exact),
  workflow_check: Type.Object({ id: id(checkIds) }, exact),
  workflow_research: Type.Object({ topic: id(researchIds) }, exact),
};
const descriptions = {
  workflow_list: 'List bounded, filtered relative filenames in the private workspace.',
  workflow_search: 'Look up a literal string in the filtered private workspace.',
  workflow_read: 'Read a permitted file in the private workspace with its expected SHA-256.',
  workflow_write: 'Replace an approved private-copy file using its expected SHA-256, or create it with expected null. Not a live write.',
  workflow_diff: 'Show the bounded private delta from baseline.',
  workflow_check: 'Run one already-approved, fixed check in a networkless copy. The host attests the result.',
  workflow_research: 'Use one of the publicResearchActions supplied in the task. The launcher provides the public query or URL. This tool accepts only an action ID.',
};

export default function tools(pi: ExtensionAPI) {
  const lease = currentRole();
  if (!lease || !Object.hasOwn(roleTools, lease.role)) throw new Error('No child tool role');
  for (const name of roleTools[lease.role] as (keyof typeof schemas)[]) {
    pi.registerTool<TSchema>({ name, label: name, description: descriptions[name], parameters: schemas[name],
      async execute(_toolId, params, signal) {
        if (currentRole() !== lease) throw new Error('Child lease ended');
        const data = await lease.invoke(name, params, signal);
        const json = JSON.stringify(data);
        const text = json.length <= 300000 ? json : JSON.stringify({ truncated: true, preview: json.slice(0, 20000) });
        return { content: [{ type: 'text', text }], details: {} };
      },
    });
  }
}
