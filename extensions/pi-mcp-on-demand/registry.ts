import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { LoadedMcpConfig, McpServerConfig } from '@earendil-works/pi-coding-agent';

export const serverName = /^[a-z][a-z0-9_-]{0,47}$/;
export const toolName = /^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/;
export const record = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v);

export function readJson(path: string, optional = false): Record<string, unknown> | undefined {
  try {
    const value: unknown = JSON.parse(readFileSync(path, 'utf8').replace(/^\uFEFF/, ''));
    if (!record(value)) throw new Error('Expected a configuration object');
    return value;
  } catch (error) {
    if (optional && (error as NodeJS.ErrnoException).code === 'ENOENT') return undefined;
    throw new Error('Missing, unreadable or invalid JSON configuration');
  }
}

export function fingerprint(value: unknown): string {
  // Order matters: native wildcard exposure rules use the first matching rule.
  return createHash('sha256').update(JSON.stringify(value)).digest('hex');
}

type Options = { agentDir: string; cwd: string; projectTrusted: boolean };
export type NativeRegistry = {
  loadMcpConfig(options: Options): LoadedMcpConfig;
  getMcpToolExposure(config: McpServerConfig, tool: string): 'direct' | 'deferred' | 'codemode' | 'hidden';
};

// Audit both raw layers, not just the merged result (a disabled override must not mask an enabled base).
export function registry(options: Options, native: NativeRegistry): LoadedMcpConfig {
  const paths = [join(options.agentDir, 'mcp.json')];
  if (options.projectTrusted) paths.push(join(options.cwd, '.pi', 'mcp.json'));
  const audit = () => paths.map((path, i) => {
    const raw = readJson(path, i > 0);
    if (raw === undefined) return null;
    if (!record(raw) || !record(raw.mcpServers)) throw new Error('Invalid MCP registry');
    const entries = Object.entries(raw.mcpServers);
    if (entries.some(([name, entry]) => !serverName.test(name) || !record(entry) || entry.enabled !== false)) {
      throw new Error('Every MCP definition and project override must explicitly disable its server');
    }
    return fingerprint(raw);
  });
  const before = audit();
  const result = native.loadMcpConfig(options);
  if (result.errors.length || result.servers.some(e => e.config.enabled !== false) || before.join() !== audit().join()) {
    throw new Error('Native MCP registry rejected or changed the configuration');
  }
  return result;
}

export function supportedTransport(config: McpServerConfig): boolean {
  if ('command' in config) return true;
  // Pi enables implicit OAuth for HTTP without Authorization. Do not involve a runtime alias in it.
  return !config.auth && !config.oauth && Object.entries(config.headers ?? {}).some(([name, value]) =>
    name.toLowerCase() === 'authorization' && value.trim().length > 0);
}
