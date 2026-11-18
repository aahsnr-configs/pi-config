# Pi Visual — Rust frontend development milestone

P04.1 provides a Ratatui chat viewport with normal Pi RPC startup, offline replay,
native Markdown, event-derived cards, scrolling and P03 permission dialogs.
Automated/PTY checks pass; actual Kitty interaction acceptance is pending.
This is a development milestone, not a production release. No Pi patches or
live configuration changes were made by development/test work.

```bash
cargo run -p pi-visual --locked -- --demo transcript
cargo run -p pi-visual --locked -- --demo transcript-long
cargo run -p pi-visual --locked -- --replay tests/fixtures/p04-transcript.jsonl
```

Replay starts no Pi process and makes no provider requests. All bundled content
is labeled synthetic/demo data. PgUp/PgDn or wheel scroll; Ctrl+End jumps to latest;
F6 switches input/transcript focus; arrows scroll when transcript-focused.
Click/drag highlights transcript rows locally. F2 steps a synthetic stream;
F3 toggles raw workflow-event details; F4 opens a local-only demo dialog;
Escape cancels that dialog; Ctrl+C exits
outside dialogs. Basic input supports end-editing, grapheme backspace and bounded
paste; long drafts show the last two wrapped rows. Ctrl+U clears the basic draft.
Enter in offline mode retains the draft and sends nothing. See
[interaction limits](docs/interaction-spec.md).

For real RPC dialogs inside this viewport, with a disposable HOME/config/project:

```bash
cargo run -p pi-visual --locked -- --rpc-fixture /absolute/path/to/pi
```

Wait for `RPC ready`, then explicitly submit `/p03-dialogs`. This loads only the
reviewed repository UI fixture. It has no provider credentials or enabled agent
tools and is not a replacement for the production permission configuration.
Do not use this diagnostic path for normal coding work. The runtime reuses
P02; rendering never launches a second backend. Direct Bash stays disabled.
Interactive session browsing and automatic reconnection are not implemented.

Replay imports are explicit, read-only, bounded regular UTF-8 JSONL files (16 MiB),
including recorded events and successful `get_state`/`get_entries` responses.
Sensitive/configuration paths are rejected; symlinks resolve to checked targets;
FIFOs are rejected without blocking. Do not export credentials into replay files.
Raw Markdown stays in the workflow record; HTML/code/links never execute or fetch.
Code wraps to cell width; Markdown tables use plain cell separators. Syntax lexing,
aligned interactive tables, clipboard copy and advanced composer features are later work.

## Normal RPC startup and recovery

Run from your coding project, or pass its directory explicitly:

```bash
/path/to/pi-visual
# From this repository, which is inside Pi configuration:
cargo run -p pi-visual --locked -- --cwd /absolute/path/to/project
# Optional explicit installed executable:
cargo run -p pi-visual --locked -- --pi /absolute/path/to/pi --cwd /absolute/path/to/project
```

Normal startup preserves Pi's usual configuration/environment/extension discovery.
The frontend never parses or logs credentials; authentication remains Pi-owned.
Startup sends only state/history requests, with no prompt or extra agent tool.
Existing backend extensions retain their own behavior. Starting Pi inside its
configuration directory is refused: use `--cwd` for a separate project.

After `RPC ready`, `/new` and `/resume <absolute-session-path>` invoke documented
session RPC commands. Active/queued work must stop first. A veto/rejection retains
the old session/view; successful transitions clear only the matching submitted
navigation command. Other unsent draft edits survive. F5 (or explicit `/restart`)
reaps/restarts the owned child and reloads its saved session; it never retries a
prompt. An unsaved conversation cannot be silently recovered, and restart is
refused if doing so would lose an in-memory transcript. The explicit no-session
`--rpc-fixture` attachment does not provide restart. Escape cancels a dialog
before global shortcuts apply. Errors retain draft/transcript with submissions
disabled until explicit recovery. A backend still owns its normal session writes.

## Scope and version policy

- **Rust**: current `stable` toolchain; workspace MSRV Rust **1.100** (released November 12, 2026). `rust-toolchain.toml` intentionally requests `stable`, not an exact compiler version. If `stable` is stale on this machine, update it manually.
- **Dependencies**: compatible SemVer requirements, not exact `=` pins. `Cargo.lock` must be committed for repeatable application builds; do not delete it to hide a dependency problem.
- **Ratatui/Crossterm**: Ratatui 0.30 release line with Crossterm 0.29 (verify the versions selected by Cargo).
- **Tokio**: the RPC library now includes an asynchronous client; runtime, macros, process, IO, sync, time and Unix signal features are requested. P04 adds the owned terminal loop and offline replay.
- **Chromium**: not necessary for building this starter. It becomes a future optional, separately sandboxed HTML/JS-renderer dependency.
- Development changes stay in the approved frontend workspace. Never start Pi in the agent configuration repository; the diagnostic uses a disposable project. No Pi patches, Pitago dependency or live settings/credential changes.

## First-time setup (recommended order)

1. Make sure no Cargo build, `rustup update`, rust-analyzer, or IDE process is concurrently updating Rustup components.
2. Install official Rustup if missing: <https://rustup.rs/>.
3. Explicitly update the toolchain and only required components, then complete that operation **before** invoking Cargo:

   ```bash
   ./scripts/first-time-rust-setup.sh
   ./scripts/doctor.sh
   ```

4. Verify stable Rust is at least 1.100 and the native linker is present. On Fedora, install `gcc` and `pkgconf-pkg-config` if necessary; native desktop libraries are **not** required for this initial TUI preview.
5. Initialize and validate the Cargo workspace:

   ```bash
   ./scripts/bootstrap.sh
   ./scripts/verify.sh
   ```

6. After success, review and commit the generated `Cargo.lock` in your project repository.

**Why bootstrap differs from CI:** bootstrap permits Cargo to generate/reconcile `Cargo.lock` **before** running `--locked` verification. CI and `verify.sh` require an already-consistent committed lockfile. Never manually edit the lockfile.

## Available local demo

```bash
cargo run -p pi-visual -- --demo home
cargo run -p pi-visual -- doctor
cargo run -p pi-visual -- --help
cargo run -p pi-visual --locked -- --probe-rpc /absolute/path/to/pi
```

The home demo is a static terminal UI; exit with `q`, Escape or Ctrl+C. No model calls or Pi process are involved.

The explicit RPC diagnostic uses a private temporary HOME/config/project, clears
inherited environment and disables extension/resource discovery. It sends only
get_state and prints no backend state or stderr. Disposable data is retained at
the reported path. Use a trusted installed Pi executable; process-group cleanup
is not OS confinement. This diagnostic does not test live security extensions or
provide normal chat. See [protocol evidence](docs/protocol-baseline.md).

## RPC connection slice

Bounded JSONL framing, `transport::probe_state` and the `--probe-rpc` diagnostic
are implemented. P04 replay and isolated live RPC attachment are implemented. See
[implementation status](docs/IMPLEMENTATION-STATUS.md) for checks and limitations.
Subprocess tests use Python 3 only as a controlled fixture.

## If Rustup errors occur

The original user's log showed (a) `quote` and `thiserror` build-script failures, and separately (b) `rust-src` download failing to rename a `.partial` file, while an old Rust 1.83 toolchain was reported. **This package does not require `rust-src`.** Such symptoms could reflect an incomplete or concurrent Rustup update, a stale installed `stable` toolchain, or another host environment issue. The build-script exit-status excerpt alone does not reveal the actual cause.

Run `./scripts/recovery-rustup.sh` for safe diagnostics. To capture the real compiler error after toolchain repair:

```bash
set -o pipefail
CARGO_BUILD_JOBS=1 cargo check --workspace --all-targets -vv 2>&1 | tee cargo-check-verbose.log
```

Never use `--no-sandbox` as a workaround for browser installation, never run `sudo cargo`, and do not delete `Cargo.lock`, global Rustup directories or all of `target` without determining why a failure occurred.

## Quality gates

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build -p pi-visual --locked
```

Framing, async transport, reducer, P03 dialogs and P04 viewport tests are implemented. See the implementation status for opt-in real-Pi and PTY results. Implement missing contracts rather than deleting or loosening tests.

## Files and status

- `crates/pi-rpc`: parsing, read-only request encoding, event preservation, response-ID correlation and offline tests.
- `crates/tui-shell`: Ratatui/Crossterm viewport, Markdown, terminal loop, basic input and modal tests.
- `scripts/`: explicit toolchain install, preflight, bootstrap, verification, doctor and Rustup recovery.
- `docs/`: architecture, protocol baseline, dependency policy, status and audit.
- `.github/workflows/ci.yml`: full locked verification after Cargo.lock is committed.

For a full implementation plan, see the separately authored master specification in your own project. No outside reference file is needed to build this starter.

## P02 library and G0 verification

`pi_rpc::client::Client` owns the async Pi session; cloned handles submit explicit
requests. Always await `shutdown()` before destroying the Tokio runtime. Watch
`Client::health` alongside events: a desynchronization requires authoritative
history reload; overflow closes the session. Timeout cancels the local waiter,
not accepted Pi work. Restart is explicit and never replays prompts.

Offline suite: `cargo test --workspace --all-targets --locked`.
Opt-in installed-Pi test (requires loopback networking; no paid inference):

```bash
PI_VISUAL_TEST_PI=/absolute/path/to/pi cargo test -p pi-rpc --test g0 --locked -- --ignored --nocapture
```

It creates private disposable HOME/config/project directories and a loopback
OpenAI-compatible fake provider. No live credentials or installed extensions
are loaded. Pi automatic networking is disabled with PI_OFFLINE. The default
suite intentionally skips this environment-dependent test. See the P02
[implementation evidence](docs/IMPLEMENTATION-STATUS.md).

## P03 isolated dialog demo

```bash
cargo run -p pi-visual --locked -- --demo dialogs /absolute/path/to/pi
python3 scripts/test-dialog-pty.py /absolute/path/to/pi
```

The demo creates a fresh private config/workspace and loads only its explicit
UI fixture. No model calls, live settings changes or tool execution. Plain y/n
answers confirm; arrows then Enter choose a select option; Escape cancels; F2
submits the multiline editor; Ctrl+G interrupts via Pi; Ctrl+C exits. Backend
editor replacements are staged when a draft exists (Ctrl+R applies). P04 also integrates these dialogs into its scrolling transcript. Revised G1 passes for permission interception only; no OS confinement is claimed; see [security evidence](docs/security.md).

Real permission-only regression (requires installed package, loopback and local stdio MCP):

```bash
PI_VISUAL_TEST_PI=/absolute/path/to/pi \
PI_VISUAL_TEST_PACKAGES=/absolute/path/to/installed/node_modules \
cargo test -p pi-rpc --test g1 --locked -- --ignored --nocapture
```

The current test requires @gotgenes/pi-permission-system42.0.0 tested interfaces,
not pi-sandbox. Original tests remain in g1_legacy_sandbox.rs as archived evidence. It never reads or copies your live policies or credentials.

P03.1 removes OS sandboxing from the intended production contract. Approved
Python/shell code and opaque MCP tools can affect paths denied to direct tools;
this is tested and documented. See [revised G1 criteria](docs/ADRs/0002-permission-only-G1.md)
for what PASS means and what protections were removed. P04 automated/PTY evidence is recorded separately.

## P04 validation and target-terminal acceptance

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --locked
python3 scripts/test-transcript-pty.py
python3 scripts/test-dialog-pty.py /absolute/path/to/pi
python3 scripts/test-dialog-pty.py /absolute/path/to/pi --p04
cargo build --release --locked -p pi-visual
./target/release/pi-visual --bench-transcript
```

The six ignored default tests require explicit real-Pi or PTY fixtures: G0,
revised G1, archived sandbox evidence, the real session lifecycle test, and two
lifecycle panic tests. The PTY
harness explicitly executes both panic tests. Archived sandbox tests are not
current acceptance requirements.

Real Kitty launch/readable output was **reported by the user**. Remaining manual
interaction acceptance is **PENDING**; unit/PTY success is not Kitty proof. In Kitty run `--demo transcript`,
`--demo transcript-long`, and the isolated `--rpc-fixture` commands above. Verify
wheel/partial scrolling, click/drag highlighting, F2 while reading older output,
40×12 through 160×50 resizing, F4/Escape with a draft, all four real dialogs,
Ctrl+C and normal shell echo/cursor restoration. Do not record this as tested
until observed. Graphics, IME and terminal-native selection behavior remain later
or manual evidence. [Status and measured benchmark](docs/IMPLEMENTATION-STATUS.md).

P04.1 isolated normal-startup/session acceptance (no provider credentials/prompts):

```bash
python3 scripts/test-session-pty.py /absolute/path/to/pi
PI_VISUAL_TEST_PI=/absolute/path/to/pi cargo test -p pi-visual session_tests --locked -- --ignored --nocapture
```

These cover normal discovery, A/B histories/branches, veto/rejection, draft,
owned-child crash/restart and rehydration. Production permission enforcement is
proven separately by revised G1, not by the UI/session fixtures. Exact human
Kitty steps and evidence boundaries: [Kitty acceptance](docs/kitty-acceptance.md).

## P04.2 ANSI status rendering

Extension status ANSI colors/styles are safely stripped as whole sequences; they
no longer appear as `[38;2;...m` or `[39m`. Existing Ratatui semantic styles remain.
The original backend string is kept in memory for diagnostics. Rebuild/relaunch
using the same normal startup commands above; no Pi/extension/config edit is needed.
Keyed status updates replace their previous values. Historical `/quit` is displayed
as history and is never re-executed during restore. Real isolated Pi/PTY regressions
cover both; post-fix genuine Kitty appearance still needs human confirmation.
