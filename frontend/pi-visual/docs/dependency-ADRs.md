# Dependency audit and policy (2026-11-16)

| Dependency | Workspace requirement | Reason |
|---|---|---|
| Rust | `stable` channel; MSRV 1.100 | Current stable; no exact toolchain pin. Explicit host update needed if stale |
| Ratatui | `0.30` | Tested API family intended for full TUI; not an exact patch pin |
| Crossterm | `0.29` | Compatible backend/input API with Ratatui 0.30 |
| serde_json | `1` | Typed-free Pi JSONL parsing already implemented |
| Tokio | 1 (tested 1.54.1) | P02 async process, IO, synchronization and timeouts |
| Chromium | Not yet installed | Opt-in browser sidecar, not a TUI build prerequisite |
| Kitty graphics crates | Not yet installed | Future compositing milestone |

Lockfile policy: commit `Cargo.lock` after resolving packages in a real Rust environment. Do not hand-author `Cargo.lock`. `bootstrap.sh` reconciles a pre-existing stale lockfile **before** `verify.sh` uses `--locked`.

Rust toolchain: `rust-toolchain.toml` contains only `channel=stable`, `profile=minimal`. It does not request `rust-src`. Install `rustfmt` and `clippy` separately, only after `rustup update stable` completes. Never run two concurrent Rustup updates.

`unsafe_code = "forbid"` applies to workspace crates explicitly inheriting workspace lints, not third-party code. Crate and CLI dependency versions are independently audited for supported API changes before future upgrades.

Official references: https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/ , https://ratatui.rs/installation/ , https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html

## P02 diagnostic slice (2026-11-16)

Tested Rust/Cargo 1.100.0, Ratatui 0.30.2, Crossterm 0.29.0 and serde_json 1.0.154. Python 3 is only a subprocess test fixture, never an application runtime.

### rustix: safe pipe polling and process-group cleanup

- Tested 1.1.6; requirement 1; features event/process plus defaults. Already cached/transitive; added directly and reconciled Cargo.lock through Cargo offline.
- Reason: review reproduced delayed timeout cleanup when a descendant retained pipes. Safe poll wrappers permit worker cancellation and safe group signaling terminates ordinary wrapper descendants. Project-owned unsafe code remains forbidden.
- Alternative: Tokio would add an async runtime and require its own cancellation design; raw libc would require unsafe code. std alone lacks these polling/signaling APIs. Removal: replace the diagnostic module if a measured async transport replaces it.
- License from installed manifest: Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT. Stable 1.x API; release-health and full transitive notices audit pending.
- Memory/binary impact NOT MEASURED. Eight MiB retained record limit plus optional CR, a two-record queue and JSON allocation overhead are not an exact RSS limit. Vec capacity can exceed retained bytes. 10 ms cancellation polling is not a real-time scheduling guarantee.
- CVE policy: retain locked resolution; review advisory/license checks and updates before release. cargo audit/deny NOT RUN; no clean vulnerability claim. Never auto-update browser or Pi.
- This is not process confinement. Trusted Pi executables only; malicious escaped descendants are outside diagnostic support.

Full licensing/advisory audits, binary/memory measurements and removal ADRs for remaining crates are still pending.

## P02 Tokio decision

Tested Tokio 1.54.1, requirement 1, features rt/macros/process/io-util/sync/time;
no network client or model runtime. Compatible 1.x maturity; MIT license from
installed manifest. bytes and tokio-macros added by Cargo from cache; lockfile
retained. Tokio owns cancellable pipe IO and child waiting; alternatives were
manual Unix polling (preserved probe only) or an additional RPC framework.
Removal boundary is client.rs; protocol/reducer do not depend on Tokio types.
Memory is limited through queue counts times record size, not exact RSS; no
measured binary-size/startup/performance claim. Advisory/transitive licensing
audits remain pending release requirements; locked updates require review and
compatibility tests, never automatic Pi updates.

## P03 dependency decision

No new packages. tui-shell now uses the already locked serde_json and Tokio
(rt/time requests, unified with pi-rpc features) to drive its isolated modal
fixture. No model-facing tools, runtime research connector or permanent Pi
extension was installed. The test-only local provider and UI extension are
explicit disposable fixtures, not runtime dependencies or user configuration.

## P04 local parser/input decisions

Tested with Rust1.100.0 and the unchanged Ratatui0.30.2/Crossterm0.29.0 API family.
Cargo reconciled Cargo.lock; no exact-version requirements or runtime research tools.

| Dependency | Tested / flags | Decision and removal boundary |
|---|---|---|
| pulldown-cmark |0.13.6; default-features=false|MIT; parser-only; GFM tables/tasks/strikethrough selected at runtime. Compatible0.13 API (pre1.0); replace markdown.rs without changing raw data. Alternative terminal Markdown wrappers add styling/compatibility coupling.|
| unicode-segmentation |1.13.3; defaults|MIT OR Apache-2.0; grapheme boundaries; replace display/editor helpers if needed; scalar iteration cannot preserve combining clusters.|
| unicode-width |0.2.3; defaults|MIT OR Apache-2.0; cell measurement compatible with Ratatui; pre1.0 upgrade reviewed; shares layout/cursor policy.|
| Tokio |1.54.1; adds signal to existing features|MIT; Linux signal shutdown; no network client/features; replace runtime signal ownership independently.|
| rustix |1.1.6; tui-shell fs plus existing RPC event/process|Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT; safe O_NONBLOCK regular-file open rejects FIFOs without blocking. Already transitive; replace only replay importer on another OS.|

Requirements are compatible SemVer ranges, lockfile carries exact tested versions.
Installed source and successful compilation override docs snippets. The tested parser version is available in the local Cargo cache; broader release-health assessment,
transitive notices, cargo audit/deny and binary/RSS deltas remain NOT MEASURED.
No vulnerability-free or license-audit completion claim. Retain locked dependencies,
review security advisories before release, and remove/disable affected optional
features rather than silently upgrading Pi or weakening isolation.
