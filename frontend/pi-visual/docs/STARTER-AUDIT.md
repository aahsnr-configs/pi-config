# Complete starter file audit (27 original entries)

The archive was audited by category, **not** falsely described as fully compiled. All 27 original paths were inspected. Three helper/recovery scripts and this audit were added to the revised package.

## Existing paths reviewed

| Original path | Review result |
|---|---|
| `.cargo/config.toml` | Retained: simple aliases and no intrusive registry flags |
| `.github/workflows/ci.yml` | Retained locked CI; added active-toolchain diagnostic |
| `.gitignore` | Retained Cargo.lock-tracked application policy |
| `AGENTS.md` | Retained no-Pi-patch/security rules; updated audit instructions |
| `CHANGELOG.md` | Added correction entry |
| `Cargo.toml` | Fixed old MSRV and exact pins; removed unused dependency |
| `LICENSE` | Retained MIT license |
| `README.md` | Fully corrected setup and failure recovery |
| `crates/pi-rpc/Cargo.toml` | Removed direct unused serde dependency |
| `crates/pi-rpc/src/lib.rs` | Retained starter parser and tests; not a full Pi RPC client |
| `crates/pi-rpc/tests/protocol.rs` | Retained original test contracts; minor style normalization by rustfmt expected |
| `crates/tui-shell/Cargo.toml` | Retained crate dependency boundaries |
| `crates/tui-shell/src/main.rs` | Retained static preview/doctor; not a live Pi process |
| `crates/tui-shell/src/ui.rs` | Retained basic UI and viewport tests |
| `docs/ADRs/0001-owned-rust-rpc-frontend.md` | Retained independently-owned Rust RPC architecture |
| `docs/IMPLEMENTATION-STATUS.md` | Updated honestly for not-run Cargo tests |
| `docs/architecture.md` | Retained scoped architecture |
| `docs/dependency-ADRs.md` | Replaced exact-pin documentation with compatible version policy |
| `docs/feature-parity.md` | Retained 23 feature families as NOT IMPLEMENTED |
| `docs/protocol-baseline.md` | Retained scoped Pi RPC baseline |
| `docs/research-sources.md` | Added Rust 1.100 source |
| `rust-toolchain.toml` | Retained stable channel; removed auto-component list |
| `rustfmt.toml` | Retained Edition 2024 formatting conventions |
| `scripts/bootstrap.sh` | Fixed stale-lock handling and added preflight |
| `scripts/doctor.sh` | Added Rustup overrides/version/path diagnostics |
| `scripts/verify.sh` | Added preflight; kept `--locked` gates |
| `tests/fixtures/rpc-get-state.jsonl` | Retained offline fixture |

## Added paths

- `scripts/toolchain-preflight.sh`
- `scripts/first-time-rust-setup.sh`
- `scripts/recovery-rustup.sh`
- `docs/STARTER-AUDIT.md`

## Important boundaries

This archive contains the original scaffold, not the user's evolved local source tree. The later test files `tests/transport.rs`, `tests/framing.rs`, and `tests/reducer.rs` were absent from the original ZIP; this revision cannot verify their signatures. The `quote`/`thiserror` build-script errors refer to crates not listed as direct dependencies in the original starter, indicating the failing local workspace already has a different dependency graph. Treat the Rustup failure and build-script failure as separate until diagnostic logs establish the cause.
