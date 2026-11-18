import { readFileSync, realpathSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";

const phaseType = "pi-prompt-optimize";
const proposal = "propose-prompt-optimization";

// Replay only the current branch, including an end whose return navigation is pending.
export function replayPhase(entries) {
  let active = null;
  let ended = null;
  for (const entry of entries) {
    if (entry.type !== "custom" || entry.customType !== phaseType) continue;
    const data = entry.data;
    if (
      data?.kind === "start" &&
      typeof data.branchStartId === "string" &&
      data.branchStartId.length &&
      Array.isArray(data.previousTools) &&
      data.previousTools.every((name) => typeof name === "string") &&
      !active
    ) {
      active = { target: data.branchStartId };
      ended = null;
    } else if (data?.kind === "end" && active) {
      ended = active;
      active = null;
    } else {
      throw new Error("Invalid optimizer phase records");
    }
  }
  return { active, ended };
}

export function registerPromptReviewGuard(pi, agentDir) {
  let sessionId;
  let lock = null;
  let fault = false;
  const packageRoot = join(agentDir, "npm/node_modules/pi-prompt-optimize");
  const entryPath = join(packageRoot, "extensions/prompt-optimize.ts");
  const skillPath = join(packageRoot, "skills/writing-for-agents/SKILL.md");

  function sync(ctx, settled = false) {
    try {
      const id = ctx.sessionManager.getSessionId();
      if (id !== sessionId) {
        sessionId = id;
        lock = null;
        fault = false;
      }
      const { active, ended } = replayPhase(ctx.sessionManager.getBranch());
      if (active || ended)
        lock = { ...(active || ended), active: Boolean(active) };
      else if (lock) lock.active = false;
      // An end record alone is insufficient: the old coding tools are restored
      // upstream before the final proposal call and its siblings have finished.
      if (
        !fault &&
        lock &&
        !lock.active &&
        ctx.sessionManager.getLeafId() === lock.target &&
        (settled || ctx.isIdle())
      )
        lock = null;
    } catch {
      fault = true;
    }
    return Boolean(lock || fault);
  }

  function status(ctx) {
    ctx.ui.setStatus(
      "prompt-review-guard",
      fault
        ? "Prompt review: invalid state; start a new session"
        : lock
          ? "Prompt review: coding tools blocked"
          : undefined,
    );
  }

  function verifiedTool(name) {
    const metadata = pi.getAllTools().find((tool) => tool.name === name);
    if (name === "read")
      return (
        metadata?.sourceInfo?.source === "builtin" &&
        metadata.sourceInfo.path === "builtin:read"
      );
    const pkg = JSON.parse(
      readFileSync(join(packageRoot, "package.json"), "utf8"),
    );
    return (
      pkg.name === "pi-prompt-optimize" &&
      pkg.version === "0.1.4" &&
      metadata?.sourceInfo?.source !== "builtin" &&
      realpathSync(metadata?.sourceInfo?.path) === realpathSync(entryPath)
    );
  }

  for (const event of [
    "session_start",
    "session_tree",
    "before_agent_start",
    "agent_settled",
  ]) {
    pi.on(event, (_event, ctx) => {
      sync(ctx, event === "agent_settled");
      status(ctx);
    });
  }

  pi.on("tool_call", (event, ctx) => {
    if (!sync(ctx)) return;
    const denied = {
      block: true,
      reason:
        "Prompt review permits only its proposal and bundled skill read. Finish /prompt-optimize end, or start a new session if recovery fails.",
    };
    if (fault || !lock?.active) return denied;
    try {
      // Verify the phase owner even for a read: an unknown optimizer version
      // must not silently inherit an old phase contract.
      if (!verifiedTool(proposal)) return denied;
      if (event.toolName === proposal) return;
      const path = event.input?.path;
      if (
        event.toolName === "read" &&
        verifiedTool("read") &&
        typeof path === "string" &&
        !path.split(/[\\/]/u).includes("..")
      ) {
        const requested = isAbsolute(path) ? path : resolve(ctx.cwd, path);
        if (realpathSync(requested) === realpathSync(skillPath)) return;
      }
    } catch {
      return denied;
    }
    return denied;
  });
}
