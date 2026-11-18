import test from "node:test";
import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { registerPromptReviewGuard, replayPhase } from "./guard.mjs";

const start = (target) => ({
  type: "custom",
  customType: "pi-prompt-optimize",
  data: {
    kind: "start",
    branchStartId: target,
    previousTools: ["read", "bash"],
  },
});
const end = {
  type: "custom",
  customType: "pi-prompt-optimize",
  data: { kind: "end" },
};
const proposal = "propose-prompt-optimization";

function fixture(t, phase = true) {
  const root = mkdtempSync(join(tmpdir(), "pi-prompt-review-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const pkg = join(root, "npm/node_modules/pi-prompt-optimize");
  mkdirSync(join(pkg, "extensions"), { recursive: true });
  mkdirSync(join(pkg, "skills/writing-for-agents"), { recursive: true });
  const entry = join(pkg, "extensions/prompt-optimize.ts");
  const skill = join(pkg, "skills/writing-for-agents/SKILL.md");
  writeFileSync(entry, "// fixture");
  writeFileSync(skill, "Fixture writing skill");
  const manifest = join(pkg, "package.json");
  writeFileSync(
    manifest,
    JSON.stringify({ name: "pi-prompt-optimize", version: "0.1.4" }),
  );
  const state = {
    entries: phase ? [start("origin")] : [],
    leaf: phase ? "working" : "origin",
    idle: true,
    id: "one",
  };
  const handlers = new Map();
  const metadata = [
    { name: "read", sourceInfo: { source: "builtin", path: "builtin:read" } },
    { name: proposal, sourceInfo: { source: "npm", path: entry } },
  ];
  const statuses = [];
  const pi = {
    on: (name, fn) => handlers.set(name, fn),
    getAllTools: () => metadata,
  };
  const ctx = {
    cwd: root,
    isIdle: () => state.idle,
    sessionManager: {
      getBranch: () => state.entries,
      getSessionId: () => state.id,
      getLeafId: () => state.leaf,
    },
    ui: { setStatus: (...value) => statuses.push(value) },
  };
  registerPromptReviewGuard(pi, root);
  const emit = (name) => handlers.get(name)({}, ctx);
  const call = (toolName, input = {}) =>
    handlers.get("tool_call")({ toolName, input }, ctx);
  return { root, skill, manifest, state, metadata, statuses, emit, call };
}

test("ordinary coding has no extra restrictions or fabricated phase", (t) => {
  const f = fixture(t, false);
  f.emit("session_start");
  assert.equal(f.call("bash"), undefined);
  assert.equal(f.call("write"), undefined);
});

test("a phase restricts tools even if upstream keeps all coding tools active", (t) => {
  const f = fixture(t);
  for (const name of [
    "bash",
    "write",
    "edit",
    "subagent",
    "codemode",
    "tool_search",
    "mcp__context7__query_docs",
    "compress",
    "decompress",
    "search_context",
  ]) {
    assert.equal(f.call(name).block, true, name);
  }
  assert.equal(f.call(proposal), undefined);
  assert.equal(f.call("read", { path: f.skill }), undefined);
});

test("only the bundled skill can be read; traversal and unrelated paths are denied", (t) => {
  const f = fixture(t);
  const other = join(f.root, "other.md");
  writeFileSync(other, "not the bundled skill");
  symlinkSync(other, join(f.root, "alias.md"));
  for (const path of [
    other,
    join(f.root, "alias.md"),
    `${f.root}/../outside`,
    "",
    undefined,
  ]) {
    assert.equal(f.call("read", { path }).block, true);
  }
  assert.equal(
    f.call("read", {
      path: "npm/node_modules/pi-prompt-optimize/skills/writing-for-agents/SKILL.md",
    }),
    undefined,
  );
});

test("unknown package versions and replaced tool implementations fail closed", (t) => {
  const f = fixture(t);
  writeFileSync(
    f.manifest,
    JSON.stringify({ name: "pi-prompt-optimize", version: "9.0.0" }),
  );
  assert.equal(f.call(proposal).block, true);
  assert.equal(f.call("read", { path: f.skill }).block, true);
  writeFileSync(
    f.manifest,
    JSON.stringify({ name: "pi-prompt-optimize", version: "0.1.4" }),
  );
  f.metadata[0].sourceInfo = { source: "npm", path: "/replacement/read.ts" };
  assert.equal(f.call("read", { path: f.skill }).block, true);
  f.metadata[1].sourceInfo.path = "/replacement/proposal.ts";
  assert.equal(f.call(proposal).block, true);
});

test("Accept/Cancel end records do not unlock remaining sibling tool calls", (t) => {
  const f = fixture(t);
  f.state.idle = false;
  f.emit("before_agent_start");
  f.state.entries.push(end);
  assert.equal(f.call("bash").block, true);
  assert.equal(f.call(proposal).block, true);
  f.state.entries = [];
  f.state.leaf = "origin";
  f.emit("session_tree");
  assert.equal(f.call("write").block, true);
  f.emit("agent_settled");
  f.state.idle = true;
  assert.equal(f.call("write"), undefined);
});

test("idle navigation after an explicit end restores normal tools", (t) => {
  const f = fixture(t);
  f.emit("session_start");
  f.state.entries.push(end);
  f.emit("agent_settled");
  assert.equal(f.call("bash").block, true);
  f.state.entries = [];
  f.state.leaf = "origin";
  f.emit("session_tree");
  assert.equal(f.call("bash"), undefined);
});

test("reload restores restrictions during active and finishing phases", (t) => {
  const f = fixture(t);
  f.emit("session_start");
  assert.equal(f.call("bash").block, true);
  const g = fixture(t);
  g.state.entries.push(end);
  g.emit("session_start");
  assert.equal(g.call("read", { path: g.skill }).block, true);
});

test("failed navigation and unrelated branches stay locked; a new session starts fresh", (t) => {
  const f = fixture(t);
  f.emit("session_start");
  f.state.entries = [];
  f.state.leaf = "different";
  f.emit("session_tree");
  assert.equal(f.call("bash").block, true);
  f.state.id = "two";
  f.emit("session_start");
  assert.equal(f.call("bash"), undefined);
});

test("malformed phase state stays locked even after records disappear", (t) => {
  const f = fixture(t);
  f.state.entries = [end];
  f.emit("session_start");
  assert.equal(f.call(proposal).block, true);
  f.state.entries = [];
  f.emit("session_tree");
  assert.equal(f.call("bash").block, true);
});

test("branch replay ignores other extensions and validates phase ordering", () => {
  assert.deepEqual(replayPhase([{ type: "custom", customType: "other" }]), {
    active: null,
    ended: null,
  });
  assert.deepEqual(replayPhase([start("a"), end, start("b")]), {
    active: { target: "b" },
    ended: null,
  });
  assert.throws(() => replayPhase([start("a"), start("b")]), /Invalid/);
  assert.throws(() => replayPhase([start("")]), /Invalid/);
});
