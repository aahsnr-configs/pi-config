import { lstatSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { getAgentDir } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';
import { here } from './host-policy.mjs';
import permissions from "../npm/node_modules/pi-permission-modes/src/index.ts";
import type { ExtensionAPI, ExtensionHandler, SessionStartEvent } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';

// The controller owns mode transitions. Upstream remains a separate tool gate.
export default function managedPermissions(pi: ExtensionAPI) {
  const baseline = readFileSync(here('permission-mode.staged.json'));
  return permissions(new Proxy(pi, {
    get(target, key) {
      if (key === "registerCommand" || key === "registerShortcut" || key === "registerFlag") return () => {};
      if (key === 'on') return (event: SessionStartEvent['type'], handler: ExtensionHandler<SessionStartEvent>) => {
        if (event !== 'session_start') return target.on(event, handler);
        return target.on('session_start', async (input, ctx) => {
          const config = join(getAgentDir(), 'permission-mode', 'permission-mode.json');
          if (lstatSync(join(ctx.cwd, '.pi'), { throwIfNoEntry: false }) || !lstatSync(config).isFile() || !readFileSync(config).equals(baseline)) {
            throw new Error('Managed permission baseline missing, changed, or overridden');
          }
          await handler(input, ctx);
          if (!ctx.sessionManager.getEntries().some(entry => entry.type === 'custom' && entry.customType === 'perm-mode' &&
              (entry.data as { mode?: string })?.mode === 'workflow-boundary')) {
            throw new Error('Reviewed permission mode was not applied');
          }
        });
      };
      if (key === "getFlag") {
        return (name: string) => {
          if (name === 'perm') return 'workflow-boundary';
          if (name === 'no-sandbox') return false;
          return target.getFlag(name);
        };
      }
      const value = Reflect.get(target, key);
      return typeof value === "function" ? value.bind(target) : value;
    },
  }));
}
