# Compatibility and scope — P04.1, 2026-11-16

The user removed pi-sandbox from intended production requirements. Its native
widget mismatch is historical and no longer blocks revised G1. No fork,
uninstall, live change or replacement of its protections was performed.

| Boundary | Verified result / limit |
|---|---|
| Pi1.5.2 | P02 regression and real four-dialog terminal PTY flow pass |
| @gotgenes/pi-permission-system42.0.0 | RPC select/input fallback; real allow/ask/deny, session/reset, path/external and MCP evidence passes |
| Native double-confirm | TUI-only; RPC uses one explicit select, not equivalent keypress behavior |
| Approved code/MCP | Downstream effects unconfined; adversarial marker mutations demonstrate this |
| Symlink policy | Pre-execution canonicalization, not race-free access |
| Timeout/abort | No backend cancel event; local guards and backend stale-ID ignoring, no exact revocation guarantee |
| Status/widget keys | Global wire keys without extension-origin attribution |
| Four-mode/emergency | No approved definitions or implementation established; unverified, not claimed |
| Frontend presentation | In-app title, keyed text cards, staged editor replacement, P04 transcript/basic input; no full P09 composer |

Installed source/version inspection does not establish the user's live enabled
configuration. No live policy/auth/session files were read. Test configurations
are explicit disposable policies with YOLO disabled. Separately approved future
permission replacements require new compatibility/security tests.

Revised G1 PASS is for the documented permission-only acceptance scope, not the
original sandbox contract. P03 is ready for acceptance on that basis. P04.1 automated/session/PTY checks
pass; human Kitty interaction remains pending. Historical original G1 PARTIAL and original24 tests are retained
under docs/history and tests/g1_legacy_sandbox.rs respectively.

Normal Pi RPC startup and explicit F5 session recovery are implemented. Tests use
private configuration and synthetic session files, not live credentials or histories.
A/B navigation, branch selection, extension veto/rejected paths, crash/restart and
stable active identity pass. Global text status/widget keys still lack origin metadata;
this does not establish per-extension attribution or OS isolation. Default backend
extension behavior remains Pi-owned. Actual installed-user configuration behavior
was not probed. Production permission evidence remains the separate G1 suite.
