#!/usr/bin/env bash
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source ./scripts/toolchain-preflight.sh
if [[ ! -s Cargo.lock ]]; then
  echo 'ERROR: missing Cargo.lock. Run ./scripts/bootstrap.sh once to generate it.' >&2
  exit 2
fi
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build -p pi-visual --locked
echo 'All Cargo quality gates passed.'
