> HISTORICAL P03 record. Superseded by approved P03.1 scope; not current requirements or recommendations.

# Verified compatibility gaps — P03, 2026-11-16

| Boundary | Verified behavior | Result |
|---|---|---|
| Pi1.5.2 RPC | select/confirm/input/editor and fire-and-forget methods supported | Typed Rust route and real four-dialog PTY tested |
| pi-sandbox0.8.2 | src/ui.ts showPermissionPrompt returns abort if ctx.mode is not tui; access-grant widget uses native setWidget factory/onTerminalInput | Denials preserved; native grants unavailable in RPC; G1 PARTIAL |
| @gotgenes/pi-permission-system42.0.0 | src/authority/permission-prompt-component.ts chooseDecisionSurface uses custom only for tui; RPC uses select/input fallback | Real Yes/No/cancel/timeout tested through Pi |
| Backend timeout/AbortSignal | dist/modes/rpc/rpc-mode.js deletes pending ID without cancel event | No exact frontend knowledge of backend resolution; local guards plus backend ignore of stale IDs |
| Extension identity | UI wire has global statusKey/widgetKey, no owner field | Separate keys preserved; same-key ownership cannot be guaranteed |

Package sources were inspected read-only under the installed npm directory.
Live settings, sandbox.json, auth stores and permission config were not read.
Installed version is not proof that an extension is enabled in live settings;
our disposable fixture explicitly loads the intended packages, separately and
together. No unconditional live-configuration compatibility claim is made.

The modal demo displays setTitle safely in-app, retains widgetPlacement but
uses a separate notice region, and stages set_editor_text if a draft exists.
Exact terminal-title mutation, widget placement and a full composer belong to
further reviewed integration work. P04 was not started.

Do not patch Pi or weaken the sandbox to satisfy G1. Keep sandbox grants
unavailable over RPC; discuss supported extension-level RPC grants with the
maintainer or obtain explicit review of a compatible replacement. Ordinary
native pi remains unchanged and available.
