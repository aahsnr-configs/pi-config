#!/usr/bin/env bash
# Read-only diagnostics for a failed rust-src download or stale toolchain.
# Does not delete cached downloads, set a default toolchain, or mutate Pi.
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
echo '=== Rustup recovery diagnostics (read-only) ==='
./scripts/doctor.sh
echo
echo '=== Suggested recovery, performed manually in order ==='
cat <<'GUIDE'
1. Stop all cargo/rustup/IDE background toolchain updates. Ensure only one
   rustup installation is running. Confirm there is free disk space.
2. Use `rustup update stable`, wait until it COMPLETES without errors.
3. If rust-src is repeatedly reinstalled even though this project does not
   require it, inspect `rustup component list --installed --toolchain stable`;
   consider `rustup component remove rust-src --toolchain stable` only after
   confirming no OTHER workflow needs rust-src. Do not delete rustup download files.
4. Run `rustup component add rustfmt clippy --toolchain stable` only after
   stable has updated successfully.
5. If rustc still reports 1.83, inspect `rustup override list`, parent
   rust-toolchain files, RUSTUP_TOOLCHAIN, PATH and distro-installed Rust.
6. From this project, run `./scripts/doctor.sh`, then `./scripts/bootstrap.sh`.
7. If quote/thiserror build scripts still fail, run
      CARGO_BUILD_JOBS=1 cargo check --workspace --all-targets -vv
   and capture the FIRST actual stderr/root-cause error. Do not assume
   an exit-status-only build-script excerpt identifies the underlying cause.
GUIDE
