#!/usr/bin/env bash
# Source this file from setup/verification scripts. No implicit toolchain installation.
# The user is responsible for serial rustup updates, outside Cargo builds.
set -Eeuo pipefail

if ! command -v rustup >/dev/null 2>&1; then
  echo 'ERROR: rustup is not installed or not in PATH.' >&2
  echo 'Install rustup from https://rustup.rs/ then restart your shell.' >&2
  exit 127
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo 'ERROR: cargo not found in PATH (install the rustup-managed stable toolchain).' >&2
  exit 127
fi
if ! command -v rustc >/dev/null 2>&1; then
  echo 'ERROR: rustc not found in PATH.' >&2
  exit 127
fi
# Starting a cargo build while rustup is changing the same toolchain can
# produce inconsistent build-script errors; resolve installs first.
rust_version="$(rustc --version 2>&1)" || {
  echo "ERROR: rustc cannot run: $rust_version" >&2
  exit 2
}
read -r _ actual_version _ <<< "$rust_version"
if [[ ! "$actual_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "ERROR: cannot parse rustc version: $rust_version" >&2
  exit 2
fi
major="${actual_version%%.*}"
rest="${actual_version#*.}"
minor="${rest%%.*}"
if (( 10#$major < 1 || (10#$major == 1 && 10#$minor < 100) )); then
  echo "ERROR: project requires Rust >=1.100, but the active compiler is $rust_version" >&2
  echo 'Run rustup update stable once, let it complete, then rerun bootstrap.' >&2
  echo 'Check rustup override list, RUSTUP_TOOLCHAIN and rust-toolchain.toml in parent directories.' >&2
  exit 2
fi
for tool in rustfmt clippy-driver; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "ERROR: missing $tool. Install components before building:" >&2
    echo '  rustup component add rustfmt clippy --toolchain stable' >&2
    exit 2
  fi
done
if ! cargo fmt --version >/dev/null 2>&1 || ! cargo clippy --version >/dev/null 2>&1; then
  echo 'ERROR: rustfmt/clippy are not available for this active toolchain.' >&2
  echo 'Run: rustup component add rustfmt clippy --toolchain stable' >&2
  exit 2
fi
printf 'Toolchain preflight OK: %s\n' "$rust_version"
