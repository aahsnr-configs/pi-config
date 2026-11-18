# Pi Visual — phase index and acceptance gates

This is a **compact phase index** distilled from the supplied master implementation specification. It is not a new implementation plan. Read only the current row and the relevant master-plan sections when executing a phase. Approved later decisions override older requirements.

| Phase | Scope | Completion evidence |
|---|---|---|
| P00 | Research, versions, requirements, threat model, architecture | Source-backed baseline and isolated workspace |
| P01 | Cargo workspace, CI, Rust quality gates and ownership | Reproducible formatter, Clippy, tests, build |
| P02 | Typed Pi RPC, JSONL transport, correlation and reducer | **G0:** real isolated Pi RPC and offline fixture tests |
| P03 | RPC extension UI, permission prompts and negative security tests | **Revised G1:** permission interception; no OS-sandbox claim |
| P04 | Ratatui viewport, virtualization, Markdown, tool cards, event loop, lifecycle | Model-free transcript replay; live integration and real Kitty where required |
| **P05** | **Kitty image protocol, multiple scrollable rectangles, partial clipping, cleanup** | **G2:** real Kitty scrolling/resizing/capture evidence; no ghost images |
| P06 | Optional isolated Chromium/CDP, frame transfer and input geometry | Browser isolation and frame transport tests |
| P07 | Interactive HTML/Canvas pendulum in a Kitty chat card | **G3:** dragging, sliders, scroll/focus/resize, security and zero UI model calls |
| P08 | Native Rust pendulum, local controls, safe spec/clock | **G4:** numerical correctness and browser-free fallback |
| P09 | Multiline composer, `/` and `@` menus, Vi mode and minimal footer | **G5:** focus, paste, editing, suggestion and modal integrity |
| P10 | Tokyo Night-inspired theme, 18 UI states, theme switching | Actual Kitty captures and V01–V18 state matrix |
| P11 | Versioned `piui/v1` widget spec, reactive state, safe expressions, adapters | **G6 initial:** GenUI/OpenUI fixture without source extensions |
| P12 | All 23 component families with interactions and galleries | 23/23 behavioral tests and real Kitty verification |
| P13 | Remaining simulations, local exports, deterministic timing | Numerical reference tests, no model calls on frames |
| P14 | Multiple advanced browser widgets, SVG/D3/Canvas and hardening | Broader G3 regression, denial/failure/offline tests |
| P15 | Extension interoperability, theme import, migration/rollback, audit | Verified approved permission architecture, no Pi config mutations |
| P16 | Token audit, property/fuzz/e2e tests, performance and real Kitty QA | **G7:** zero incremental provider calls; **G8:** production maturity |
| P17 | Release packaging, reproducible binaries, licenses, docs, upgrades | Candid release readiness matrix and maintainer guide |

## Gate interpretations

- **G0:** official Pi RPC correctness and process lifecycle using disposable configuration.
- **G1 (approved revision):** genuine `pi-permission-system` interception, fail-closed dialogs, negative tests, no independent tool bypass. This does **not** demonstrate kernel/OS sandbox containment.
- **G2:** multiple Kitty images in a scrolling transcript, resizing, clipping, cleanup and fallback.
- **G3:** truly interactive, sandboxed Chromium-backed HTML/Canvas *inside* Kitty; static screenshots cannot pass.
- **G4:** native numerical pendulum fallback.
- **G5:** dependable composer/autocomplete/focus and input recovery.
- **G6:** functional local widget-schema adaptation, without requiring original GenUI/OpenUI extensions.
- **G7:** measured zero incremental model calls and model-visible tool overhead for local UI actions.
- **G8:** integrated release-quality reliability, security, performance, test and QA evidence.

## Current handoff

The user reported P00–P03 complete or revised as documented. P04 has passing automated checks and demonstrated live Kitty RPC startup; manual Kitty input/layout acceptance and any pending P04.1/P04.2 findings must be resolved according to the **current** `docs/IMPLEMENTATION-STATUS.md` before promoting it to PASS. This paragraph is a handoff note, not proof of acceptance; update it as actual evidence changes.

Keep the full master implementation specification available in the repository if you have it, but do not paste or reread it wholesale on every routine change. For a new phase, consult its relevant sections on demand.
