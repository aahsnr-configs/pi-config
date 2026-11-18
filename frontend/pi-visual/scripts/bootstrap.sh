#!/usr/bin/env bash
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
source ./scripts/toolchain-preflight.sh

# Only this project is touched; no Pi credentials or user settings are read.
if [[ ! -s Cargo.lock ]]; then
  echo 'Generating the application Cargo.lock via Cargo (network may be required).'
  cargo generate-lockfile
else
  echo 'Checking Cargo.lock against current manifests (normal first-run resolution permitted).'
  # Important: DO NOT use --locked until Cargo had the opportunity to
  # reconcile a changed manifest with an older lockfile.
  cargo metadata --format-version=1 --no-deps > /dev/null
  cargo check --workspace --all-targets
fi

# Run rustfmt before the read-only formatting check.
cargo fmt --all
./scripts/verify.sh
echo 'Bootstrap OK. Commit Cargo.lock after reviewing the resolved dependencies.'
