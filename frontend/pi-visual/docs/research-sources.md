# Initial research sources (2026-11-16)

- Official Pi 1.5.2 RPC commands/events/extension UI: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc-commands.md
- Official Pi extension UI limitations in RPC: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc-extension-ui.md
- Ratatui installation/version/MSRV: https://ratatui.rs/installation/
- Ratatui basic terminal lifecycle: https://docs.rs/ratatui/0.30.2/ratatui/fn.run.html
- Cargo lockfile policy: https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
- Kitty graphics protocol (future): https://sw.kovidgoyal.net/kitty/graphics-protocol/

Only tagged/documented sources are requirements. Model-driven research tools (Exa/Context7) are optional during future research and never runtime dependencies.

- Rust 1.100 release: https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/

## Verified for P02 diagnostic, 2026-11-16

- Tagged Pi JSONL/correlation: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc.md ; installed docs/rpc.md confirms strict LF/optional CR.
- Tagged CLI: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/cli.md ; actual --help verified with temporary HOME/config.
- Rust 1.100 blocking buffering API: https://doc.rust-lang.org/std/io/trait.BufRead.html ; fill_buf/consume used rather than unbounded read_until allocation.
- rustix 1.1.6 actual installed src/event/poll.rs and src/process/kill.rs override snippets. Context7 resolved /bytecodealliance/rustix and queried poll/process-group APIs; https://docs.rs/rustix/1.1.6/rustix/event/fn.poll.html and https://docs.rs/rustix/1.1.6/rustix/process/fn.kill_process_group.html .
- Exa was not needed for this bounded diagnostic; no research tools or model-facing schemas were installed at runtime. Remaining mandated comparative/browser/graphics research stays pending P00.

## P02 async implementation, 2026-11-16

Installed Pi1.5.2 docs/json.md, rpc-commands.md, models.md, custom-provider.md and
environment-variables.md supplied exact event and endpoint/offline semantics.
Tagged source: https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/json.md
and https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/models.md .
Context7 /tokio-rs/tokio/tokio-1.54.1 queried for process/IO cancellation; actual
cached Tokio1.54.1 API compiled/tested. No research service is a runtime dependency.

## P03.1 source verification,2026-11-16

Installed permission-system42.0.0 config-schema.ts, permission-dialog.ts,
permission-prompt-component.ts, wrapper-analysis.ts, canonicalize-path.ts and
docs/configuration.md supplied policy/RPC/interpreter-limit semantics.
Installed Pi1.5.2 docs/mcp.md supplied mcp.json, builtin:mcp loading and
mcp__server__tool direct exposure. No live configuration was read and no
research connector/runtime dependency was added.

## P04 local Markdown and terminal APIs,2026-11-16

Context7 resolved /pulldown-cmark/pulldown-cmark for targeted parser event/option
lookup; actual cached0.13.6 Tag/TagEnd/Event types govern implemented code.
Primary version documentation: https://docs.rs/pulldown-cmark/0.13.6/pulldown_cmark/ .
Ratatui0.30.2 Frame/TestBackend/try_init source and Crossterm0.29.0 event/paste/
keyboard-mode source were inspected and exercised through compilation and PTYs:
https://docs.rs/ratatui/0.30.2/ratatui/ and
https://docs.rs/crossterm/0.29.0/crossterm/event/ .
No broad Exa query was needed for this bounded local implementation. No provider,
CDN, browser download or runtime docs connector is involved.

## P04.1 installed semantics,2026-11-16

Installed Pi1.5.2 docs/cli.md (--session),rpc-commands.md (cancelled transition
responses,get_state/get_entries),session-format.md (v3 tree fixtures),extensions.md
(session_before_switch veto) and json.md (retry/settled fields) govern this slice.
Tagged authoritative links:
https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/rpc-commands.md
and https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/session-format.md .
No new library API/dependency or broad web discovery was needed; installed sources
and real isolated process behavior were checked directly. Rust1.100 Clippy's
as_chunks diagnostic was applied without suppressing the lint.

## P04.2 installed ANSI producers,2026-11-16

Read-only source trace: installed pi-extmgr0.4.2 src/utils/status.ts:102 calls
ctx.ui.theme.fg("dim",...) before setStatus("extmgr",...). Installed pi-lens4.5.0
dist/index.js:159992–160009 builds success/error/dim theme strings for pi-lens-lsp.
Installed Pi1.5.2 dist/modes/rpc/rpc-mode.js:105–113 serializes statusText unchanged.
No live configuration,credentials or session payloads were inspected or exported.
User-provided RGB/ANSI observations plus these sources informed synthetic fixtures;
no runtime Exa/Context7,dependency update or unapproved provider call was needed.
