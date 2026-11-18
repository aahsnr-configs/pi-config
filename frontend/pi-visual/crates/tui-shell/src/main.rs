//! Initial development milestone only.
//! `--demo home` draws a local, non-agent Tokyo Night layout; it does not connect to Pi.

mod app;
mod dialog_demo;
mod launch;
mod markdown;
mod modal;
mod session;
#[cfg(test)]
mod session_tests;
mod shell;
mod terminal;
mod transcript;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{
    env, io,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--version" => {
            println!(
                "pi-visual {} (development scaffold)",
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }
        [flag] if flag == "doctor" => {
            doctor();
            Ok(())
        }
        [flag, executable] if flag == "--probe-rpc" => run_rpc_probe(executable),
        [demo, mode] if demo == "--demo" && mode == "transcript" => shell::demo(),
        [demo, mode] if demo == "--demo" && mode == "transcript-long" => shell::long_demo(),
        [flag, path] if flag == "--replay" => shell::replay_file(path),
        [flag] if flag == "--bench-transcript" => {
            shell::benchmark();
            Ok(())
        }
        [flag, exe] if flag == "--rpc-fixture" => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(async {
                let client = dialog_demo::fixture_client(exe, true).await?;
                shell::run_live(client).await
            })
        }
        [demo, mode] if demo == "--demo" && mode == "home" => run_home_demo(),
        [demo, mode, exe] if demo == "--demo" && mode == "dialogs" => dialog_demo::run(exe),
        _ => {
            let launch = launch::Launch::parse(&args)?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(shell::start(launch))
        }
    }
}

fn print_help() {
    println!(
        "pi-visual — Rust frontend development scaffold\n\n\
         Usage:\n  pi-visual [--pi <executable>] [--cwd <project>]\n  pi-visual doctor\n  pi-visual --demo home\n  pi-visual --demo transcript\n  pi-visual --demo transcript-long\n  pi-visual --replay <authorized-jsonl>\n  pi-visual --rpc-fixture <absolute-pi-executable>\n  pi-visual --bench-transcript\n  pi-visual --demo dialogs <absolute-pi-executable>\n  pi-visual --probe-rpc <absolute-pi-executable>\n  pi-visual --version\n  pi-visual --help\n\n\
         The RPC probe uses disposable configuration and sends only get_state.\n\
         Offline replay makes no RPC/model calls. Normal startup loads Pi normally and sends no prompt until explicit input. Never run it inside Pi configuration; use --cwd <project>."
    );
}

/// Explicit diagnostic only: never use this isolated setup for normal chat,
/// where Pi must retain the user's existing extensions and security policies.
fn run_rpc_probe(executable: &str) -> io::Result<()> {
    use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt, path::Path};
    if !Path::new(executable).is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Pi executable must be an absolute path",
        ));
    }
    let executable = Path::new(executable).canonicalize()?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let root = env::temp_dir().join(format!("pi-visual-rpc-{}-{nonce}", std::process::id()));
    // Atomic creation; never reuse a pre-existing or symlinked probe directory.
    DirBuilder::new().mode(0o700).create(&root)?;
    for name in ["home", "project", "agent", "config", "cache"] {
        DirBuilder::new().mode(0o700).create(root.join(name))?;
    }
    println!("Disposable probe data retained at {}", root.display());
    let mut command = Command::new(executable);
    command
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("PI_CODING_AGENT_DIR", root.join("agent"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("TERM", "dumb")
        .current_dir(root.join("project"))
        .args([
            "--mode",
            "rpc",
            "--no-session",
            "--no-extensions",
            "--no-mcp",
            "--no-skills",
            "--no-prompt-templates",
            "--no-themes",
            "--no-context-files",
        ]);
    let report = pi_rpc::transport::probe_state(&mut command, Duration::from_secs(10))
        .map_err(io::Error::other)?;
    println!(
        "PASS: correlated get_state; child {} reaped; {} unrelated and {} malformed records",
        report.pid, report.ignored_records, report.rejected_records
    );
    println!("No prompt, steer, follow_up or bash sent. Backend state and stderr are not printed.");
    Ok(())
}

fn doctor() {
    println!("pi-visual {} — local preflight", env!("CARGO_PKG_VERSION"));
    println!("Pi API request parser: available (offline)");
    if let Ok(value) = pi_rpc::read_only_request("doctor-check", "get_state") {
        println!("Pi get_state JSONL encoder: {value}");
    }
    for (program, arg) in [
        ("rustc", "--version"),
        ("cargo", "--version"),
        ("pi", "--version"),
        ("kitty", "--version"),
    ] {
        match Command::new(program).arg(arg).output() {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout);
                let first_line = version.lines().next().unwrap_or("(empty)");
                println!("{program}: {first_line}");
            }
            Ok(_) => println!("{program}: installed, unable to report version"),
            Err(_) => println!("{program}: unavailable"),
        }
    }
    println!("Chrome/Kitty graphics: NOT PROBED (planned separate security/capability gate)");
    println!("No Pi RPC subprocess started; no credentials read; no inference calls.");
}

fn run_home_demo() -> io::Result<()> {
    let directory = env::current_dir()?.to_string_lossy().into_owned();
    // `ratatui::run` restores the terminal on the ordinary exit path and on panic.
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| ui::draw_home(frame, &directory))?;
            match event::read()? {
                Event::Key(key) => {
                    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    })
}
