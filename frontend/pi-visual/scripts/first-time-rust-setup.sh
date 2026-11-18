#!/usr/bin/env bash
# Optional explicit toolchain setup. Do not run concurrently with Cargo,
# another rustup, rust-analyzer auto-install, or an IDE toolchain update.
set -Eeuo pipefail
if ! command -v rustup >/dev/null 2>&1; then
  echo 'ERROR: install rustup from https://rustup.rs/ before running this script.' >&2
  exit 127
fi
printf 'Updating stable Rust toolchain (this may download files).\n'
rustup update stable
printf 'Adding only the needed formatting/linting components.\n'
rustup component add rustfmt clippy --toolchain stable
printf 'rust-src is intentionally NOT installed.\n'
rustup run stable rustc --version
