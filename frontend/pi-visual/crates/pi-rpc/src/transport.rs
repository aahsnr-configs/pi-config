//! A one-shot, read-only compatibility probe, not a live chat transport.
//!
//! Caller supplies an explicitly isolated Pi command. No shell, environment
//! discovery, credentials, model calls or automatic restart is implemented here.
//! Only use with a trusted backend configured without extensions. Cleanup kills
//! its process group and reaps the direct child. This is not OS confinement;
//! a hostile executable could escape the group and is outside this probe's scope.

use crate::{
    Incoming, RpcResponse, encode_line,
    framing::{JsonlReader, ReadError},
    read_only_request,
};
use std::{
    error::Error,
    fmt,
    io::{self, BufReader, Read, Write},
    os::{fd::AsFd, unix::process::CommandExt},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const REQUEST_ID: &str = "pi-visual-state-probe";

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeError {
    InvalidTimeout,
    Spawn,
    Reader,
    Write,
    Timeout,
    Eof,
    MismatchedCommand,
    Rejected,
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidTimeout => "RPC probe requires a positive, representable timeout",
            Self::Spawn => "RPC probe could not start the backend",
            Self::Reader => "RPC probe reader failed",
            Self::Write => "RPC probe could not write get_state",
            Self::Timeout => "RPC probe timed out",
            Self::Eof => "RPC backend closed stdout before a matching response",
            Self::MismatchedCommand => "RPC response ID matched a different command",
            Self::Rejected => "RPC backend rejected get_state",
        })
    }
}

impl Error for ProbeError {}

#[derive(Debug)]
pub struct ProbeReport {
    pub response: RpcResponse,
    pub pid: u32,
    pub ignored_records: usize,
    pub rejected_records: usize,
}

struct ProbeChild {
    child: Child,
    stop: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl Drop for ProbeChild {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Close stdin, terminate the direct child, then reap it on every path.
        self.child.stdin.take();
        // The unreaped child reserves this PID, so its group ID cannot have
        // been reused by an unrelated process between spawn and cleanup.
        if let Some(pid) = rustix::process::Pid::from_raw(self.child.id() as i32) {
            let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Poll before every pipe read so inherited descriptors cannot strand a worker
/// during cleanup, even if a descendant escapes the child's process group.
struct CancellablePipe<R> {
    pipe: R,
    stop: Arc<AtomicBool>,
}

impl<R: Read + AsFd> Read for CancellablePipe<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        if bytes.is_empty() {
            return Ok(0);
        }
        let timeout = Timespec {
            tv_sec: 0,
            tv_nsec: 10_000_000,
        };
        loop {
            if self.stop.load(Ordering::Acquire) {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "RPC reader stopped",
                ));
            }
            let mut fds = [PollFd::new(&self.pipe, PollFlags::IN)];
            match poll(&mut fds, Some(&timeout)) {
                Ok(0) => continue,
                Ok(_) => return self.pipe.read(bytes),
                Err(rustix::io::Errno::INTR) => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }
}

fn deliver(
    sender: &SyncSender<Result<Incoming, ReadError>>,
    stop: &AtomicBool,
    mut record: Result<Incoming, ReadError>,
) -> bool {
    while !stop.load(Ordering::Acquire) {
        match sender.try_send(record) {
            Ok(()) => return true,
            Err(TrySendError::Disconnected(_)) => return false,
            Err(TrySendError::Full(pending)) => {
                record = pending;
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
    false
}

/// Send exactly one `get_state` request and reap the backend after the response.
///
/// Both child output streams are drained independently; stderr is discarded,
/// never buffered or returned. The stdout queue holds at most two records plus
/// the reader's current record. Unrelated events and malformed records do not
/// extend the absolute deadline. This does not exercise model or tool execution.
pub fn probe_state(command: &mut Command, timeout: Duration) -> Result<ProbeReport, ProbeError> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .filter(|_| !timeout.is_zero())
        .ok_or(ProbeError::InvalidTimeout)?;
    let child = command
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ProbeError::Spawn)?;
    let mut guard = ProbeChild {
        child,
        stop: Arc::new(AtomicBool::new(false)),
        workers: Vec::new(),
    };
    let stdout = guard.child.stdout.take().ok_or(ProbeError::Reader)?;
    let stderr = guard.child.stderr.take().ok_or(ProbeError::Reader)?;
    let (sender, receiver) = mpsc::sync_channel(2);
    let stop = Arc::clone(&guard.stop);
    guard.workers.push(
        thread::Builder::new()
            .name("pi-rpc-stdout".into())
            .spawn(move || {
                let pipe = CancellablePipe {
                    pipe: stdout,
                    stop: Arc::clone(&stop),
                };
                let mut reader = JsonlReader::new(BufReader::new(pipe));
                while !stop.load(Ordering::Acquire) {
                    let record = match reader.next_record() {
                        Ok(Some(record)) => Ok(record),
                        Ok(None) => break,
                        Err(error) => Err(error),
                    };
                    let terminal =
                        matches!(record, Err(ReadError::Io(_) | ReadError::TruncatedRecord));
                    if !deliver(&sender, &stop, record) || terminal {
                        break;
                    }
                }
            })
            .map_err(|_| ProbeError::Reader)?,
    );
    let stop = Arc::clone(&guard.stop);
    guard.workers.push(
        thread::Builder::new()
            .name("pi-rpc-stderr".into())
            .spawn(move || {
                let mut stderr = CancellablePipe { pipe: stderr, stop };
                let _ = io::copy(&mut stderr, &mut io::sink());
            })
            .map_err(|_| ProbeError::Reader)?,
    );

    let request = read_only_request(REQUEST_ID, "get_state").map_err(|_| ProbeError::Write)?;
    let line = encode_line(&request).map_err(|_| ProbeError::Write)?;
    guard
        .child
        .stdin
        .as_mut()
        .ok_or(ProbeError::Write)?
        .write_all(line.as_bytes())
        .map_err(|_| ProbeError::Write)?;

    let mut ignored_records: usize = 0;
    let mut rejected_records: usize = 0;
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(ProbeError::Timeout)?;
        match receiver.recv_timeout(remaining) {
            Err(mpsc::RecvTimeoutError::Timeout) => return Err(ProbeError::Timeout),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(ProbeError::Eof),
            Ok(Err(ReadError::Io(_))) => return Err(ProbeError::Reader),
            Ok(Err(ReadError::TruncatedRecord)) => return Err(ProbeError::Eof),
            Ok(Err(_)) => rejected_records = rejected_records.saturating_add(1),
            Ok(Ok(Incoming::Response(response))) if response.id.as_deref() == Some(REQUEST_ID) => {
                if response.command != "get_state" {
                    return Err(ProbeError::MismatchedCommand);
                }
                if !response.success {
                    return Err(ProbeError::Rejected);
                }
                return Ok(ProbeReport {
                    response,
                    pid: guard.child.id(),
                    ignored_records,
                    rejected_records,
                });
            }
            Ok(Ok(_)) => ignored_records = ignored_records.saturating_add(1),
        }
    }
}
