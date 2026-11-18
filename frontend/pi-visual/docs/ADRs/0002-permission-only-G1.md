# ADR 0002 — P03.1 permission-only G1

Date: 2026-11-16. Status: accepted scope change explicitly requested by the user.
This ADR supersedes the sandbox requirements in the original master/P03 contract;
it does not change Pi, live configuration, or future browser isolation requirements.

## Decision and original requirements

Production Pi Visual uses @gotgenes/pi-permission-system (tested 42.0.0), or a
future replacement only after separate approval. pi-sandbox is not a production
dependency or compatibility requirement. No compatibility fork is to be created,
maintained or recommended. Nothing is uninstalled from the user's live Pi setup.

Original G1 required real installed sandbox and permission enforcement, denied
writes with unchanged filesystem state, real interactive approval handling, and
native sandbox grant-widget compatibility over RPC. The latter left original
G1 PARTIAL. Original P03 records are retained under docs/history/P03-original-*;
the full original 24-case test is archived in tests/g1_legacy_sandbox.rs. It is
not part of revised acceptance and is not run by --test g1. All old assertions
are retained; no production code or previously passing tests were discarded.

## Protections removed from the intended production contract

- pi-sandbox's OS-mediated tool subprocess filesystem/process/network confinement.
- Native sandbox grant-widget parity and combined-extension interoperability.
- Any claim that an approved command remains confined to configured paths, or
  that permission interception confines MCP servers/extensions/frontend/browser.

These are removals, not equivalent replacements. Permissions remain host tool
interception, not mandatory access control. Browser execution stays disabled.

## Revised G1 acceptance criteria

1. Existing P02 transport and typed extension UI remain tested; no independent
   frontend host shell, automatic approval, or retries of denied tool calls.
2. Actual installed permission extension under unmodified Pi enforces tested
   allow/ask/deny rules via RPC. Denial, cancel, timeout and stale responses do
   not authorize the rejected intercepted operation.
3. Known path-bearing builtin operations and visible Bash operands honor tested
   path and external-directory policies, including sensitive fixture paths,
   symlinks resolved before execution and traversal. This is not a TOCTOU claim.
4. Session approval supports repeated allowed operations, respects a separate
   denied path, and does not survive a new Pi process at the same fixture paths.
5. Real stdio MCP dispatch is tested for allow/deny/path-deny/ask; denied calls
   never reach the fixture tool. This is not proof about remote servers or every
   deferred/codemode adapter.
6. Adversarial approved Python, shell-script and opaque MCP execution explicitly
   demonstrates unrestricted downstream effects on otherwise denied markers.
   Rejecting those calls preserves markers. No isolation guarantee follows from
   a passing interception test. Explicit approval delegates opaque execution.
7. Test roots/configs/models/markers are disposable, no live credentials/policies
   are read or changed, no paid provider is used, and Cargo regressions pass.
8. Four-mode/emergency requirements remain conditional. No definitions or
   implementation were found in the approved repository specification/ADRs or
   installed package schema. The master required four modes only *if configured*;
   that is not a defined four-mode design. No such feature is claimed. YOLO is
   disabled and is not treated as an emergency override. A later explicit design
   would need independent implementation/acceptance; an outside specification
   cannot be inferred or silently waived.

A PASS under this ADR means tested permission interception plus documented
limitations, not universal filesystem denial or OS sandbox equivalence. The
original G1 remains historically PARTIAL and is not retroactively passed.

## Verification

Revised real integration: 34 scenarios, including an additional process restart
inside the session-grant scenario. Default suite retains all P02/P03 tests.
Exact commands/outcomes: docs/IMPLEMENTATION-STATUS.md. Astra reviewed the scope,
fixtures and limits; its external-directory evidence gap was closed by separate
external read/write allow/deny and ask tests with path policy otherwise allowing.
