/**
 * OpenCode-inspired, display-only UI for Pi 1.5.2.
 * Native input, completion, Markdown, tools and permission UI remain host-owned.
 * No model-facing registrations, messages, requests, subprocesses or disk IO.
 */
import { homedir } from "node:os";
import {
  CustomEditor,
  type ExtensionAPI,
  type ExtensionContext,
  type KeybindingsManager,
  type Theme,
  type ThemeBg,
  type ThemeColor,
  type ToolRendererResolver,
} from "@earendil-works/pi-coding-agent";
import {
  Box,
  Text,
  isAppleTerminalSession,
  replaceControlCharacters,
  rgbColor,
  styleText,
  truncateToWidth,
  visibleWidth,
  type Component,
  type EditorTheme,
  type TUI,
} from "@earendil-works/pi-tui";

const DEFAULTS = {
  placeholder: "Ask anything…  \"Fix a TODO in the codebase\"",
  maxActivities: 128,
  minHomeRows: 18,
  maxSpacerRows: 64,
  rail: "┃",
} as const;
const DOCK = "opencode-ui:dock";
const ACTIVITY = "opencode-ui:activity";

// SafeText — decorative labels only; native content is never rewritten.
export function safeLabel(value: string, limit = 160): string {
  return replaceControlCharacters(value.slice(0, 2048))
    .replace(/[\u202a-\u202e\u2066-\u2069]/g, "")
    .replace(/\s+/g, " ").trim().slice(0, limit);
}
function fit(text: string, width: number): string {
  return truncateToWidth(text, Math.max(0, Math.floor(width)));
}
function pad(text: string, width: number): string {
  const clipped = fit(text, width);
  return clipped + " ".repeat(Math.max(0, width - visibleWidth(clipped)));
}

// ThemeAdapter — query the active theme on every render, never cache its colors.
export class ThemeAdapter {
  constructor(readonly current: () => Theme) {}
  fg(role: ThemeColor, text: string): string { return this.current().fg(role, text); }
  bg(role: ThemeBg, text: string): string {
    const theme = this.current();
    // Resolve even terminal-default roles to a concrete fill. No opacity logic.
    return theme.style(text, { bg: theme.colors[role] });
  }
}

// TerminalGeometry — public APIs do not report other extensions' dock heights.
// Position the composer's first row at two-thirds of the viewport. Its native
// height includes completions, so their appearance consumes spacer, not input.
export function spacerRows(rows: number, editorRows = 3, footerRows = 1): number {
  if (!Number.isFinite(rows) || rows < DEFAULTS.minHomeRows) return 0;
  const height = Math.floor(rows);
  return Math.max(0, Math.min(DEFAULTS.maxSpacerRows,
    height - Math.floor(height * 2 / 3) - editorRows - footerRows,
    height - editorRows - footerRows - 1));
}
class TerminalGeometry {
  editorRows = 3;
  footerRows = 1;
}

// SessionPhase — branch metadata is not conversation activity.
export class SessionPhase {
  value: "home" | "chat" | "shutdown" = "shutdown";
  reconcile(ctx: ExtensionContext): void {
    this.value = ctx.sessionManager.getBranch().some(entry =>
      ["message", "custom_message", "compaction", "branch_summary"].includes(entry.type),
    ) ? "chat" : "home";
  }
  commit(): void { if (this.value !== "shutdown") this.value = "chat"; }
}

// Original Pi brand pixels (4 columns × 2 rows), expressed using public colour
// primitives. Brand colours match the stock logo; decorative colours follow the
// active theme. No import from Pi's private pi-logo module.
export function stockPiLogo(theme: Theme, textFallback = isAppleTerminalSession()): string[] {
  const mode = theme.getColorMode();
  const coral = rgbColor(228, 138, 122);
  const blue = rgbColor(79, 142, 179);
  const yellow = rgbColor(234, 182, 93);
  if (textFallback) return [styleText("P", { fg: coral }, mode) + styleText("i", { fg: yellow }, mode)];
  return [styleText("▀", { fg: coral, bg: blue }, mode) + styleText("▀█", { fg: coral }, mode) + " ",
    styleText("█▀", { fg: blue }, mode) + " " + styleText("█", { fg: yellow }, mode)];
}

// Dashboard — custom header only; startup diagnostics remain in Pi's transcript.
class Dashboard implements Component {
  constructor(private phase: SessionPhase, private styles: ThemeAdapter, private tui: TUI) {}
  render(width: number): string[] {
    if (this.phase.value !== "home" || this.tui.terminal.rows < 22 || width < 24) return [];
    const logo = stockPiLogo(this.styles.current());
    const frameWidth = 18;
    const left = " ".repeat(Math.max(0, Math.floor((width - frameWidth) / 2)));
    const border = (text: string) => this.styles.fg("borderMuted", text);
    const lines = [border("╭────── ✦ ───────╮"),
      ...logo.map(line => border("│") + " ".repeat(6) + pad(line, 4) + " ".repeat(6) + border("│")),
      border("╰────────────────╯")];
    return [...Array<string>(Math.max(0, Math.floor(this.tui.terminal.rows * 0.35) - 2)).fill(""),
      ...lines.map(line => left + line), ""];
  }
  invalidate(): void {}
}
class AdaptiveDock implements Component {
  constructor(private phase: SessionPhase, private tui: TUI, private geometry: TerminalGeometry) {}
  render(): string[] {
    return this.phase.value === "home" && this.tui.mode === "fullscreen"
      ? Array<string>(spacerRows(this.tui.terminal.rows, this.geometry.editorRows, this.geometry.footerRows)).fill("") : [];
  }
  invalidate(): void {}
}

// NativeCompletionAdapter — SAFE NATIVE BELOW. No list copies, scanners, input
// interception, overlay focus, private geometry or pointer-coordinate remapping.
export const NativeCompletionAdapter = { placement: "native-below" } as const;

// OpenCodeEditor — protected border hooks keep every native row/column intact.
// The metadata occupies the bottom border, inside the tinted composer surface.
// A second minimum input row would change native pointer geometry; do not add it.
export class OpenCodeEditor extends CustomEditor {
  private bottom = "";
  constructor(tui: TUI, editorTheme: EditorTheme, keys: KeybindingsManager,
    private styles: ThemeAdapter, private metadata: () => string,
    private measure: (height: number) => void = () => {}) {
    super(tui, editorTheme, keys, { paddingX: 1 });
  }
  setPaddingX(padding: number): void {
    // Pi reapplies its configured padding after constructing custom editors.
    // Reserve one native padding cell for the rail, with native cursor math.
    super.setPaddingX(Math.max(1, padding));
  }
  protected renderTopBorder(width: number, hidden: number): string {
    // Keep real scroll indicators, without changing the native row geometry.
    const line = hidden > 0 || width < 2 ? super.renderTopBorder(width, hidden)
      : this.borderColor("╭" + "─".repeat(width - 2) + "╮");
    return this.styles.bg("toolPendingBg", line);
  }
  protected renderBottomBorder(width: number, hidden: number): string {
    this.bottom = this.styles.bg("toolPendingBg",
      this.styles.fg("accent", "╰") + pad(this.styles.fg("muted", ` ${safeLabel(this.metadata())}${hidden > 0 ? `  ↓ ${hidden}` : ""} `), Math.max(0, width - 2))
      + (width > 1 ? this.styles.fg("borderMuted", "╯") : ""));
    return width <= 0 ? "" : fit(this.bottom, width);
  }
  render(width: number): string[] {
    if (width <= 0) return [];
    const lines = super.render(width);
    const bottomIndex = lines.indexOf(fit(this.bottom, width));
    if (bottomIndex < 1) { this.measure(lines.length); return lines; }
    for (let i = 1; i < bottomIndex; i++) {
      let line = lines[i]!;
      // Replace only native left padding: no extra cell, offset or wrap change.
      if (this.getPaddingX() > 0 && width > 2 && line.startsWith(" ")) {
        line = this.styles.fg("accent", DEFAULTS.rail) + line.slice(1);
      }
      if (i === 1 && width > 5 && this.getText() === "" && !this.isShowingAutocomplete()) {
        // Keep the real cursor's complete ANSI/IME marker and its original cell.
        const cursorEnd = Math.min(width, Math.max(0, this.getPaddingX()) + 1);
        const available = width - cursorEnd;
        const trailingSpaces = line.match(/ +$/)?.[0].length ?? 0;
        // Column slicing at the cursor boundary can discard Pi's zero-width
        // fake-cursor closing marker. Replace only the known trailing blanks.
        if (available > 0 && trailingSpaces >= available) {
          line = line.slice(0, line.length - available) + pad(this.styles.fg("dim", ` ${DEFAULTS.placeholder}`), available - 1) + " ";
        }
      }
      // Replace only a spare native right-padding cell, never a cursor/content cell.
      if (width > 2 && line.endsWith(" ")) line = line.slice(0, -1) + this.styles.fg("borderMuted", "│");
      lines[i] = this.styles.bg("toolPendingBg", line);
    }
    this.measure(lines.length); // includes native completion rows, when visible
    return lines;
  }
}

// CardPrimitives — genuine themed Box cards on extension-owned surfaces.
export class PanelCard implements Component {
  constructor(readonly content: Component, private styles: ThemeAdapter,
    private role: ThemeColor = "accent", private background: ThemeBg = "toolPendingBg") {}
  render(width: number): string[] {
    if (width < 6) return this.content.render(Math.max(1, width)).map(line => fit(line, width));
    const box = new Box(1, 0, text => this.styles.bg(this.background, text));
    box.addChild(this.content);
    const edge = (left: string, right: string) => this.styles.bg(this.background,
      this.styles.fg(this.role, left) + this.styles.fg("borderMuted", "─".repeat(width - 2) + right));
    return [edge("╭", "╮"),
      ...box.render(width - 2).map(line => this.styles.bg(this.background,
        this.styles.fg(this.role, DEFAULTS.rail) + line + this.styles.fg("borderMuted", "│"))),
      edge("╰", "╯")];
  }
  invalidate(): void { this.content.invalidate(); }
}

// ToolViewResolver — compose native static Text call components, retaining their
// complete content, state, caches and expansion. Interactive/custom components,
// self-rendered shells, unknown tools and missing renderers pass through.
const CARD_TOOLS = new Set(["read", "ls", "find", "grep", "write", "bash"]);
export function toolViewResolver(): ToolRendererResolver {
  return (name, next) => {
    const existing = next();
    if (!CARD_TOOLS.has(name) || !existing?.renderCall || existing.renderShell === "self") return existing;
    const renderCall = existing.renderCall;
    return {
      ...existing,
      renderCall(args, theme, context) {
        const previous = context.lastComponent;
        const component = renderCall(args, theme, {
          ...context, lastComponent: previous instanceof PanelCard ? previous.content : previous,
        });
        // Native Text and its static subclasses have no input/focus ownership.
        // Never wrap a component that exposes keyboard or pointer handlers.
        const interactive = component.handleInput !== undefined || component.handleMouse !== undefined;
        if (interactive || !(component instanceof Text)) return component;
        const category: ThemeColor = name === "read" ? "mdLink" : "accent";
        const state = context.isError ? "error" : name === "bash" && context.executionStarted && !context.isPartial
          ? "success" : category;
        return new PanelCard(component, new ThemeAdapter(() => theme), state);
      },
    };
  };
}

// WorkflowStore — bounded, read-only facts; never retains tool arguments/results.
export interface ActivityRecord {
  id: string;
  name: string;
  parent?: string;
  state: "running" | "succeeded" | "failed" | "interrupted";
  durationMs?: number;
}
export class WorkflowStore {
  readonly records = new Map<string, ActivityRecord>();
  partial = false;
  running = false;
  waiting = 0;
  failed = false;
  reset(): void { this.records.clear(); this.partial = false; this.running = false; this.waiting = 0; this.failed = false; }
  start(id: string, name: string, parent?: string): void {
    if (this.records.has(id)) return;
    if (this.records.size >= DEFAULTS.maxActivities) {
      // Evict oldest settled record first. In pathological all-active workloads
      // retain a bounded subset; never invent a success for the omitted calls.
      const oldest = [...this.records.values()].find(record => record.state !== "running");
      if (!oldest) this.partial = true;
      this.records.delete(oldest?.id ?? this.records.keys().next().value!);
    }
    this.records.set(id, { id, name: safeLabel(name, 60), parent, state: "running" });
  }
  end(id: string, isError: boolean, durationMs?: number): void {
    const record = this.records.get(id);
    if (!record) return; // includes late callbacks from discarded branches
    record.state = isError ? "failed" : "succeeded";
    record.durationMs = durationMs;
  }
  settle(aborted: boolean): void {
    this.running = false;
    this.waiting = 0;
    for (const record of this.records.values()) {
      if (record.state === "running") record.state = "interrupted";
    }
    if (aborted) this.failed = true;
  }
}
class ActivityView implements Component {
  constructor(private store: WorkflowStore, private styles: ThemeAdapter, private tui: TUI) {}
  render(width: number): string[] {
    if (!this.store.running || this.store.waiting || this.tui.terminal.rows < 22) return [];
    const active = [...this.store.records.values()].filter(record => record.state === "running");
    if (active.length === 0) return []; // native thinking/retry indicator already exists
    const nested = active.filter(record => record.parent !== undefined).length;
    const title = `${active.length} ${this.store.partial ? "tracked " : ""}active tool${active.length === 1 ? "" : "s"}${nested ? ` (${nested} nested)` : ""}`;
    return new PanelCard(new Text(this.styles.fg("muted", title), 0, 0), this.styles).render(width);
  }
  invalidate(): void {}
}

// MarkdownPresentation — intentional no-op. Pi's native Markdown owns headings,
// code, tables, lists and links; no transformer or custom assistant messages.

// FooterView — current directory; cost then context, and nothing else.
export function displayDirectory(cwd: string, home: string): string {
  const value = cwd === home ? "~" : cwd.startsWith(home + "/") ? "~" + cwd.slice(home.length) : cwd;
  return safeLabel(value, 2048);
}
export function sessionCost(ctx: ExtensionContext): number | undefined {
  // Public subscription metadata only; never query credentials or a provider.
  if (!ctx.model || ctx.model.provider === "kimi-coding" || ctx.modelRegistry.isUsingOAuth(ctx.model)) return undefined;
  let cost = 0;
  let known = false;
  for (const entry of ctx.sessionManager.getEntries()) {
    const usage = entry.type === "usage" ? entry.usage
      : (entry.type === "compaction" || entry.type === "branch_summary") ? entry.usage
      : entry.type === "message" && (entry.message.role === "assistant" || entry.message.role === "toolResult") ? entry.message.usage
      : undefined;
    if (usage && Number.isFinite(usage.cost?.total) && usage.cost.total >= 0) {
      cost += usage.cost.total;
      known = true;
    }
  }
  return known ? cost : undefined;
}
export class FooterView implements Component {
  constructor(private context: () => ExtensionContext, private styles: ThemeAdapter,
    private cost: () => number | undefined, private measure: (height: number) => void = () => {}) {}
  render(width: number): string[] {
    if (width <= 0) return [];
    const ctx = this.context();
    const usage = ctx.getContextUsage();
    const window = usage?.contextWindow ?? ctx.model?.contextWindow;
    const windowText = window ? `${Math.round(window / 1000)}k` : "?";
    const percent = usage?.percent;
    const context = `Context ${percent === undefined || percent === null ? "?" : Math.round(percent) + "%"} / ${windowText}`;
    const amount = this.cost();
    const cost = amount === undefined ? "Cost —" : `Cost $${amount.toFixed(3)} est`;
    const cwd = displayDirectory(ctx.cwd, homedir());
    const right = `${cost}  ·  ${context}`;
    let lines: string[];
    if (width >= visibleWidth(right) + 8) {
      const left = fit(cwd, width - visibleWidth(right) - 2);
      lines = [pad(left, width - visibleWidth(right)) + right];
    } else {
      // On narrow terminals preserve the basename and context before cost.
      const basename = cwd.split("/").pop() || cwd;
      const left = fit(basename, Math.max(1, width - visibleWidth(context) - 2));
      lines = visibleWidth(context) + visibleWidth(left) + 1 <= width
        ? [pad(left, width - visibleWidth(context)) + context]
        : [fit(basename, width), fit(context, width)];
    }
    this.measure(lines.length);
    return lines.map(line => this.styles.fg("dim", line));
  }
  invalidate(): void {}
}

// SettingsGuard / Diagnostics — no config writes, no repeated ownership race.
class Diagnostics {
  private shown = new Set<string>();
  warn(ctx: ExtensionContext, message: string): void {
    if (this.shown.has(message)) return;
    this.shown.add(message);
    ctx.ui.notify(`opencode-ui: ${message}`, "warning");
  }
}
function SettingsGuard(ctx: ExtensionContext): boolean { return ctx.hasUI && ctx.mode === "tui"; }

// LifecycleCoordinator — only observational handlers, all returning undefined.
export default function opencodeUI(pi: ExtensionAPI): void {
  const phase = new SessionPhase();
  const store = new WorkflowStore();
  const geometry = new TerminalGeometry();
  const diagnostics = new Diagnostics();
  let current: ExtensionContext | undefined;
  let tui: TUI | undefined;
  let cachedCost: number | undefined;
  let installed = false;
  let ownHeader = false;
  let ownFooter = false;
  let cleaning = false;
  let conflict = false;
  let factory: ReturnType<ExtensionContext["ui"]["getEditorComponent"]>;
  const styles = new ThemeAdapter(() => current!.ui.theme);
  const update = (ctx: ExtensionContext): void => {
    if (!installed) return;
    current = ctx;
    if (conflict) diagnostics.warn(ctx, "another extension replaced a UI slot; it will not be reclaimed until a new session.");
    tui?.requestRender();
  };
  const dispose = (ctx: ExtensionContext): void => {
    phase.value = "shutdown";
    store.reset();
    if (!installed) return;
    cleaning = true;
    ctx.ui.setWidget(DOCK, undefined);
    ctx.ui.setWidget(ACTIVITY, undefined);
    if (ctx.ui.getEditorComponent() === factory) ctx.ui.setEditorComponent(undefined);
    if (ownHeader) ctx.ui.setHeader(undefined);
    if (ownFooter) ctx.ui.setFooter(undefined);
    ownHeader = ownFooter = installed = false;
    tui = undefined;
    current = undefined;
    cleaning = false;
  };
  const commit = (ctx: ExtensionContext): void => { phase.commit(); update(ctx); };
  // Only use a call-header card where the native header carries no extra data
  // that would be lost. Bash retains its command, exit status and test evidence.
  pi.registerToolRenderer(toolViewResolver());
  pi.on("session_start", (_event, ctx) => {
    if (!SettingsGuard(ctx)) return;
    if (installed) dispose(ctx);
    if (ctx.ui.getEditorComponent()) {
      diagnostics.warn(ctx, "custom editor already installed; UI left native to avoid replacing its owner.");
      return;
    }
    current = ctx;
    phase.reconcile(ctx);
    store.reset();
    cachedCost = sessionCost(ctx);
    conflict = false;
    // A widget factory exposes the public TUI without changing focus.
    ctx.ui.setWidget(DOCK, host => {
      tui = host;
      return new AdaptiveDock(phase, host, geometry);
    }, { placement: "belowEditor" });
    if (!tui || tui.mode !== "fullscreen") {
      ctx.ui.setWidget(DOCK, undefined);
      diagnostics.warn(ctx, "fullscreen mode is required; no editor/header/footer replaced.");
      current = undefined;
      tui = undefined;
      return;
    }
    installed = true;
    factory = (host, editorTheme, keys) => new OpenCodeEditor(host, editorTheme, keys, styles,
      () => `${ctx.ui.getEditorText().trimStart().startsWith("!") ? "Shell" : "Chat"}  ·  ${safeLabel(current?.model?.name ?? current?.model?.id ?? "No model", 32)}  ·  ${current?.thinkingLevel ?? pi.getThinkingLevel()}`,
      height => { geometry.editorRows = height; });
    ctx.ui.setEditorComponent(factory);
    ownHeader = true;
    ctx.ui.setHeader(host => Object.assign(new Dashboard(phase, styles, host), {
      dispose() { ownHeader = false; if (!cleaning) conflict = true; },
    }));
    ownFooter = true;
    ctx.ui.setFooter(() => Object.assign(new FooterView(() => current!, styles, () => cachedCost,
      height => { geometry.footerRows = height; }), {
      dispose() { ownFooter = false; if (!cleaning) conflict = true; },
    }));
    ctx.ui.setWidget(ACTIVITY, host => new ActivityView(store, styles, host));
    update(ctx);
  });
  pi.on("session_shutdown", (_event, ctx) => { dispose(ctx); });
  pi.on("session_tree", (_event, ctx) => {
    if (!installed) return;
    phase.reconcile(ctx);
    store.reset();
    cachedCost = sessionCost(ctx);
    update(ctx);
  });
  // Input hooks run before later extensions can consume a local UI command.
  // Commit only once Pi confirms actual agent or shell activity instead.
  pi.on("user_bash", (_event, ctx) => { commit(ctx); });
  pi.on("before_agent_start", (_event, ctx) => { commit(ctx); });
  pi.on("agent_start", (_event, ctx) => { store.running = true; store.failed = false; commit(ctx); });
  pi.on("tool_execution_start", (event, ctx) => {
    if (!installed) return;
    store.start(event.toolCallId, event.toolName, event.parentToolCallId);
    commit(ctx);
  });
  pi.on("tool_execution_end", (event, ctx) => {
    store.end(event.toolCallId, event.isError, event.durationMs);
    update(ctx);
  });
  pi.on("message_end", (event, ctx) => {
    if (!installed) return;
    if (event.message.role === "assistant" && (event.message.stopReason === "error" || event.message.stopReason === "aborted")) store.failed = true;
    cachedCost = sessionCost(ctx);
    update(ctx);
  });
  pi.on("agent_settled", (event, ctx) => {
    store.settle(event.aborted);
    if (installed) cachedCost = sessionCost(ctx);
    update(ctx);
  });
  pi.on("ui_prompt_start", (_event, ctx) => { store.waiting++; update(ctx); });
  pi.on("ui_prompt_end", (_event, ctx) => { store.waiting = Math.max(0, store.waiting - 1); update(ctx); });
  pi.on("session_compact", (_event, ctx) => { if (installed) cachedCost = sessionCost(ctx); update(ctx); });
  pi.on("model_select", (_event, ctx) => { if (installed) cachedCost = sessionCost(ctx); update(ctx); });
  pi.on("thinking_level_select", (_event, ctx) => { update(ctx); });
}
