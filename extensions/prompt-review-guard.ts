import type { ExtensionAPI } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js';
import { getAgentDir } from '../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/config.js';
import { registerPromptReviewGuard } from '../prompt-review/guard.mjs';

export default function guard(pi: ExtensionAPI) {
  registerPromptReviewGuard(pi, getAgentDir());
}
