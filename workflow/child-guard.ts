import type { ExtensionAPI, ExtensionContext } from "../install/releases/1.5.2/node_modules/@earendil-works/pi-coding-agent/dist/index.js";
import { currentRole, roleTools } from "./roles.mjs";

type Lease = {
  role: string;
  model: string;
  workspace: string;
  childBound?: Readonly<{ role: string; model: string; workspace: string; tools: readonly string[] }>;
  canWrite(path: unknown): boolean;
};

const modelId = (ctx: ExtensionContext) => ctx.model && `${ctx.model.provider}/${ctx.model.id}`;

export default function guard(pi: ExtensionAPI) {
  let lease: Lease | undefined;
  const permitted = new Set<string>();
  const active = (ctx: ExtensionContext) => lease && currentRole() === lease && ctx.cwd === lease.workspace && modelId(ctx) === lease.model;

  pi.on("session_start", (_event, ctx) => {
    lease = undefined;
    permitted.clear();
    const activeNames = new Set(pi.getActiveTools());
    pi.setActiveTools([]);
    const candidate = currentRole() as Lease | undefined;
    if (!candidate || candidate.childBound || !Object.hasOwn(roleTools, candidate.role)) {
      throw new Error("Missing exclusive workflow role lease");
    }
    if (ctx.cwd !== candidate.workspace || modelId(ctx) !== candidate.model) {
      throw new Error("Child workspace or model differs from the approved launch");
    }
    const required: string[] = [...roleTools[candidate.role], "structured_output"];
    const registered = new Set(pi.getAllTools().map(tool => tool.name));
    for (const name of required) {
      if (!registered.has(name) || !activeNames.has(name)) throw new Error(`Required workflow tool unavailable: ${name}`);
    }
    pi.setActiveTools(required);
    const actual = pi.getActiveTools();
    if (actual.length !== required.length || !required.every(name => actual.includes(name))) {
      pi.setActiveTools([]);
      throw new Error("Child tool allowlist did not apply");
    }
    for (const name of required) permitted.add(name);
    candidate.childBound = Object.freeze({ role: candidate.role, model: candidate.model,
      workspace: candidate.workspace, tools: Object.freeze([...required]) });
    lease = candidate;
  });

  pi.on("tool_call", (event, ctx) => {
    if (!lease || !active(ctx)) return { block: true, reason: "No active workflow role" };
    if (!permitted.has(event.toolName)) {
      return { block: true, reason: "Tool denied by workflow role policy" };
    }
    if (event.toolName === "workflow_write" && !lease.canWrite(event.input.path)) {
      return { block: true, reason: "Write is outside the approved scope" };
    }
  });

  pi.on("session_before_switch", () => ({ cancel: true }));
  pi.on("session_before_fork", () => ({ cancel: true }));
  pi.on("session_before_tree", () => ({ cancel: true }));
  pi.on("session_shutdown", () => { lease = undefined; });
  pi.on("user_bash", () => ({
    result: { output: "Shell is unavailable in managed roles.", exitCode: 1, cancelled: false, truncated: false },
  }));
}
