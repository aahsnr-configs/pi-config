#!/usr/bin/env bash
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
printf 'Pi Visual toolchain diagnostics — read-only; no Pi/model calls\n\n'
printf 'Working directory: %s\n' "$PWD"
for tool in rustup rustc cargo rustfmt clippy-driver cc pi kitty chromium chromium-browser; do
  if command -v "$tool" >/dev/null 2>&1; then
    printf 'available: %-18s %s\n' "$tool" "$(command -v "$tool")"
  else
    printf 'missing:   %s\n' "$tool"
  fi
done
printf '\n'
if command -v rustup >/dev/null 2>&1; then
  rustup --version || true
  rustup toolchain list || true
  rustup override list || true
  rustup show active-toolchain || true
fi
if command -v rustc >/dev/null 2>&1; then rustc --version || true; fi
if command -v cargo >/dev/null 2>&1; then cargo --version || true; fi
if command -v pi >/dev/null 2>&1; then pi --version || true; fi
if command -v kitty >/dev/null 2>&1; then kitty --version || true; fi
printf 'RUSTUP_TOOLCHAIN=%s\n' "${RUSTUP_TOOLCHAIN:-<unset>}"
printf 'RUSTUP_HOME=%s\n' "${RUSTUP_HOME:-<unset>}"
printf 'CARGO_HOME=%s\n' "${CARGO_HOME:-<unset>}"
printf 'RUSTFLAGS=%s\n' "${RUSTFLAGS:-<unset>}"
printf 'RUSTC_WRAPPER=%s\n' "${RUSTC_WRAPPER:-<unset>}"
printf 'TERM=%s\n' "${TERM:-<unset>}"
if [[ -s Cargo.lock ]]; then echo 'Cargo.lock: exists'; else echo 'Cargo.lock: missing (expected before first bootstrap)'; fi
printf 'No configuration was modified.\n'
