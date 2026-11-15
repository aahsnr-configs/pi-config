/**
 * Adaptive prompt placement for Pi 1.5.2+ (including pi-fast-tui).
 *
 * Fresh, empty fullscreen session: place the composer around the upper-middle.
 * First submitted prompt or agent run: restore the normal bottom dock.
 *
 * Deliberately does not replace the editor, footer, header, or renderer.
 * Adds no LLM tools, context, messages, timers, network calls, or subprocesses.
 */
import type {
  ExtensionAPI,
  ExtensionContext,
} from "@earendil-works/pi-coding-agent";

const WIDGET_ID = "adaptive-prompt-position";

/** Adjust these constants to tune the initial vertical position. */
export const SPACER_RATIO = 0.42;
export const MIN_TERMINAL_ROWS = 22;
export const RESERVED_TERMINAL_ROWS = 16;
export const MAX_SPACER_ROWS = 34;

/** Pure, bounded calculation; also useful for automated testing. */
export function spacerRows(terminalRows: number): number {
  if (!Number.isFinite(terminalRows) || terminalRows < MIN_TERMINAL_ROWS) {
    return 0;
  }

  const rows = Math.floor(terminalRows);
  return Math.max(
    0,
    Math.min(
      MAX_SPACER_ROWS,
      Math.floor(rows * SPACER_RATIO),
      rows - RESERVED_TERMINAL_ROWS,
    ),
  );
}

/**
 * New-session detection follows the ACTIVE branch, not the full JSONL file.
 * Metadata alone (model switches, labels, session names) does not count as a turn.
 * Summaries and custom context messages do, even when old messages were pruned.
 */
function hasConversation(ctx: ExtensionContext): boolean {
  return ctx.sessionManager.getBranch().some((entry) => {
    switch (entry.type) {
      case "message":
      case "custom_message":
      case "compaction":
      case "branch_summary":
        return true;
      default:
        return false;
    }
  });
}

function isInteractive(ctx: ExtensionContext): boolean {
  return ctx.hasUI && ctx.mode === "tui";
}

export default function adaptivePrompt(pi: ExtensionAPI): void {
  let visible = false;
  let installed = false;

  // Using a factory bypasses Pi's 10-line cap for string-array widgets.
  // Using tui.mode ensures regular mode stays unaffected, even after a
  // runtime mode switch. Reading terminal.rows on each render also makes
  // resize and Kitty window splits work without polling or timers.
  const install = (ctx: ExtensionContext): void => {
    if (!isInteractive(ctx) || installed) return;

    ctx.ui.setWidget(
      WIDGET_ID,
      (tui) => ({
        render(_width: number): string[] {
          if (!visible || tui.mode !== "fullscreen") return [];
          return Array<string>(spacerRows(tui.terminal.rows)).fill("");
        },
        invalidate(): void {},
      }),
      { placement: "belowEditor" },
    );
    installed = true;
  };

  const clear = (ctx: ExtensionContext): void => {
    visible = false;
    if (!installed) return;
    installed = false;
    if (isInteractive(ctx)) ctx.ui.setWidget(WIDGET_ID, undefined);
  };

  const reconcile = (ctx: ExtensionContext): void => {
    if (!isInteractive(ctx)) return;
    if (hasConversation(ctx)) {
      clear(ctx);
    } else {
      visible = true;
      install(ctx);
    }
  };

  // Pi 1.5.2 emits session_start after startup, reload, new, resume or fork.
  // It uses session_shutdown before replacements; no session_switch event.
  pi.on("session_start", (_event, ctx) => {
    clear(ctx);
    reconcile(ctx);
  });

  pi.on("session_shutdown", (_event, ctx) => {
    clear(ctx);
  });

  // A /tree navigation can select an entirely empty branch.
  pi.on("session_tree", (_event, ctx) => {
    reconcile(ctx);
  });

  // 'input' runs on submission, not on each keystroke. Commands like
  // /fast-tui that Pi intercepts earlier do not trigger this handler.
  pi.on("input", (event, ctx) => {
    if (!visible || event.source !== "interactive") return;
    if (event.text.trim().length === 0 && (event.images?.length ?? 0) === 0) return;
    clear(ctx);
  });

  // Shell submissions are also real activity, even when excluded from LLM context.
  pi.on("user_bash", (_event, ctx) => {
    if (visible) clear(ctx);
  });

  // Fallbacks handle prompts submitted by another extension or startup flags.
  pi.on("before_agent_start", (_event, ctx) => {
    if (visible) clear(ctx);
  });
  pi.on("agent_start", (_event, ctx) => {
    if (visible) clear(ctx);
  });
}
