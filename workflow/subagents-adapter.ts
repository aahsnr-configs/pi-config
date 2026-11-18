import subagents from '../npm/node_modules/pi-subagents/bundle/index.js';
import type { ExtensionAPI } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';

// The controller uses only the structured foreground delegation event bridge.
// No user-facing commands/shortcuts/flags are offered by this adapter.
export default function managedSubagents(pi: ExtensionAPI) {
  return subagents(new Proxy(pi, {
    get(target, key) {
      if (key === 'registerCommand' || key === 'registerShortcut' || key === 'registerFlag') return () => {};
      const value = Reflect.get(target, key);
      return typeof value === 'function' ? value.bind(target) : value;
    },
  }));
}
