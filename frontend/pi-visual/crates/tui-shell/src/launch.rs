//! Normal backend startup. No frontend credential/config parsing or shell execution.
use pi_rpc::client::{Client, Limits};
use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};
#[derive(Clone)]
pub struct Launch {
    pub executable: OsString,
    pub cwd: PathBuf,
}
impl Launch {
    pub fn parse(args: &[String]) -> io::Result<Self> {
        let mut executable = OsString::from("pi");
        let mut cwd = std::env::current_dir()?;
        let (fields, remainder) = args.as_chunks::<2>();
        for pair in fields {
            match pair[0].as_str() {
                "--pi" => executable = pair[1].clone().into(),
                "--cwd" => cwd = pair[1].clone().into(),
                _ => return Err(io::Error::other("unknown option; use --help")),
            }
        }
        if !remainder.is_empty() {
            return Err(io::Error::other("--pi and --cwd require values"));
        }
        let cwd = cwd.canonicalize()?;
        if !cwd.is_dir() {
            return Err(io::Error::other("--cwd must be a project directory"));
        }
        for config in [
            std::env::var_os("PI_CODING_AGENT_DIR").map(PathBuf::from),
            std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".pi/agent")),
        ]
        .into_iter()
        .flatten()
        {
            let config = config.canonicalize().unwrap_or(config);
            if cwd.starts_with(config) {
                return Err(io::Error::other(
                    "refusing Pi startup inside its configuration directory; use --cwd <project>",
                ));
            }
        }
        // Relative executable paths must not change meaning when the child cwd changes.
        if Path::new(&executable).components().count() > 1 && Path::new(&executable).is_relative() {
            executable = std::env::current_dir()?.join(&executable).into_os_string();
        }
        Ok(Self { executable, cwd })
    }
    pub fn command(&self, session: Option<&str>) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.executable);
        command.current_dir(&self.cwd);
        if let Some(path) = session {
            command.arg("--session").arg(path);
        }
        // Client adds --mode rpc. Preserve Pi's normal environment and security
        // extension discovery; the frontend never enumerates or logs secrets.
        command
    }
    pub fn spawn(&self) -> io::Result<Client> {
        Client::spawn_ordered(
            &mut self.command(None),
            Limits {
                record_bytes: pi_rpc::MAX_RPC_LINE_BYTES,
                event_capacity: 64,
                ..Default::default()
            },
        )
        .map_err(|e| io::Error::other(format!("Cannot launch Pi RPC ({e}); check --pi and --cwd")))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_command_retains_security_discovery_and_has_no_prompt() {
        let launch = Launch {
            executable: "/trusted/pi".into(),
            cwd: "/tmp".into(),
        };
        let command = launch.command(Some("/tmp/saved session.jsonl"));
        let args: Vec<_> = command.as_std().get_args().collect();
        assert_eq!(args, vec!["--session", "/tmp/saved session.jsonl"]);
        assert_eq!(command.as_std().get_current_dir(), Some(Path::new("/tmp")));
        assert_eq!(command.as_std().get_envs().count(), 0);
        assert!(Launch::parse(&["--cwd".into()]).is_err());
    }
}
