# Changelog

## Unreleased — initial scaffold, 2026-11-16

- Added independently owned Rust workspace and minimal TUI preview.
- Added read-only Pi RPC JSONL parser, request correlation and offline tests.
- Added Cargo bootstrap, verification scripts, CI, architectural decision and phase register.
- No release, production-ready implementation or live Pi RPC validation is claimed.

## Audited setup correction — 2026-11-16

- Raised minimum Rust to 1.100; stable toolchain remains rolling.
- Removed exact package pins and unused direct serde dependency.
- Added serial Rustup setup and diagnostic/recovery scripts.
- Fixed bootstrap lockfile generation/reconciliation before locked checks.
- No Cargo compilation is claimed in the authoring environment.
