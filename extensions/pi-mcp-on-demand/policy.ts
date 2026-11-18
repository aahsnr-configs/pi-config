import { record, serverName, toolName } from './registry.ts';

type Action = 'allow' | 'ask' | 'deny';
type AllowedRule = {
  action: 'allow' | 'ask';
  roles: ['parent'];
  definitionSha256: string;
  tools: Map<string, Action>;
};
export type Rule = { aliases: string[] } & ({ action: 'deny' } | AllowedRule);
export type Policy = { enabled: boolean; maxActiveServers: number; readyTimeoutMs: number; servers: Map<string, Rule> };
const action = (x: unknown): x is Action => x === 'allow' || x === 'ask' || x === 'deny';
const keys = (value: Record<string, unknown>, allowed: string[]) => Object.keys(value).every(k => allowed.includes(k));
const alias = /^[a-zA-Z][a-zA-Z0-9_-]*(?: [a-zA-Z0-9][a-zA-Z0-9_-]*){0,4}$/;

export function parsePolicy(raw: Record<string, unknown> | undefined): Policy {
  const invalid = () => new Error('Invalid on-demand policy (expected strict version 1)');
  if (!raw || !keys(raw, ['version', 'enabled', 'maxActiveServers', 'readyTimeoutMs', 'servers']) || raw.version !== 1 ||
      typeof raw.enabled !== 'boolean' || typeof raw.maxActiveServers !== 'number' || !Number.isInteger(raw.maxActiveServers) ||
      raw.maxActiveServers < 1 || raw.maxActiveServers > 4 || typeof raw.readyTimeoutMs !== 'number' ||
      !Number.isInteger(raw.readyTimeoutMs) || raw.readyTimeoutMs < 100 || raw.readyTimeoutMs > 15_000 || !record(raw.servers)) throw invalid();
  const servers = new Map<string, Rule>();
  const names = new Map(Object.keys(raw.servers).map(name => [name, name]));
  for (const [name, entry] of Object.entries(raw.servers)) {
    if (!serverName.test(name) || !record(entry) || !action(entry.action) ||
        !Array.isArray(entry.aliases) || entry.aliases.length > 8) throw invalid();
    const aliases: string[] = [];
    for (const value of entry.aliases) {
      if (typeof value !== 'string' || value.length > 64 || !alias.test(value)) throw invalid();
      const lower = value.toLowerCase();
      if ((names.has(lower) && names.get(lower) !== name) || aliases.includes(lower)) throw invalid();
      names.set(lower, name);
      aliases.push(lower);
    }
    if (entry.action === 'deny') {
      if (!keys(entry, ['action', 'aliases'])) throw invalid();
      servers.set(name, { action: 'deny', aliases });
      continue;
    }
    if (!keys(entry, ['action', 'aliases', 'roles', 'definitionSha256', 'tools']) || !Array.isArray(entry.roles) ||
        entry.roles.length !== 1 || entry.roles[0] !== 'parent' || typeof entry.definitionSha256 !== 'string' ||
        !/^[a-f0-9]{64}$/.test(entry.definitionSha256) || !record(entry.tools)) throw invalid();
    const tools = new Map<string, Action>();
    for (const [tool, decision] of Object.entries(entry.tools)) {
      if (!toolName.test(tool) || !action(decision)) throw invalid();
      tools.set(tool, decision);
    }
    servers.set(name, { action: entry.action, aliases, roles: ['parent'], definitionSha256: entry.definitionSha256, tools });
  }
  return { enabled: raw.enabled, maxActiveServers: raw.maxActiveServers, readyTimeoutMs: raw.readyTimeoutMs, servers };
}
