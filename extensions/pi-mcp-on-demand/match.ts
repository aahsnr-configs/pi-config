import type { Policy } from './policy.ts';

export type Request = { kind: 'none' } | { kind: 'ambiguous' } | { kind: 'selected'; names: string[] };
const none: Request = { kind: 'none' };
const ambiguous: Request = { kind: 'ambiguous' };

// Intentionally small grammar. Excerpts, template invocations and embedded directives do not
// authorize anything. Use an uninterrupted directive prefix for more complex prompts.
export function resolvePrompt(prompt: string, policy: Policy): Request {
  if (prompt.length > 32_768 || /^(?:[ \t]*\r?\n)*(?: {4}|\t)/.test(prompt)) return none;
  const text = prompt.trimStart();
  if (/\[no-mcp\]/i.test(text) || /\b(?:no|not|never|without|don['’]t|avoid|skip|except|instead)\b/i.test(text)) return none;
  if (/^\[use-mcp:/i.test(text)) {
    const names = new Set<string>();
    let rest = text;
    while (/^\[use-mcp:/i.test(rest)) {
      const directive = /^\[use-mcp:([a-z][a-z0-9_-]{0,47})\](?=\s|$)/i.exec(rest);
      if (!directive) return ambiguous;
      const name = directive[1].toLowerCase();
      if (!policy.servers.has(name)) return ambiguous;
      names.add(name);
      rest = rest.slice(directive[0].length).trimStart();
    }
    if (/\[use-mcp:/i.test(rest)) return ambiguous;
    return { kind: 'selected', names: [...names] };
  }
  const request = /^(?:please[ ,]+)?(?:use|connect to) +/i.exec(text);
  if (!request) return none;
  let rest = text.slice(request[0].length).toLowerCase().trimEnd();
  const mentions = [...policy.servers].flatMap(([name, rule]) => [name, ...rule.aliases].map(alias => ({ name, alias })));
  mentions.sort((a, b) => b.alias.length - a.alias.length);
  const names = new Set<string>();
  while (true) {
    const match = mentions.find(({ alias }) => rest.startsWith(alias) && /^(?:[ ,.!?:;]|$)/.test(rest.slice(alias.length)));
    if (!match) return names.size ? ambiguous : none;
    names.add(match.name);
    rest = rest.slice(match.alias.length);
    const conjunction = /^(?: *, *(?:and +)?| +and +)/.exec(rest);
    if (!conjunction) break;
    rest = rest.slice(conjunction[0].length);
  }
  if (!/^(?:[.!]?\s*| +(?:to|for) +[^\r\n]+|[.:;] +[^\r\n]+)$/.test(rest) || /\[use-mcp:/.test(rest)) return ambiguous;
  return { kind: 'selected', names: [...names] };
}
