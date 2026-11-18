//! Python is only a controlled subprocess fixture, never a frontend runtime.
use pi_rpc::transport::{ProbeError, probe_state};
use std::{
    process::Command,
    time::{Duration, Instant},
};

fn fixture(source: &str) -> Command {
    let mut command = Command::new("python3");
    command.env_clear().args(["-u", "-c", source]);
    command
}

#[test]
fn correlates_state_after_interleaved_records_and_drains_stderr() {
    let mut child = fixture(
        r#"
import json,sys,os
r=json.loads(sys.stdin.readline())
assert r == {'id':'pi-visual-state-probe','type':'get_state'}
sys.stderr.write('sensitive diagnostic\n' * 20000)
sys.stderr.flush()
print('invalid json')
print(json.dumps({'type':'agent_start'}))
print(json.dumps({'type':'response','id':'other','command':'get_state','success':True}))
print(json.dumps({'type':'response','id':r['id'],'command':'get_state','success':True,'data':{'fixture':True}}))
sys.stdin.read()
"#,
    );
    let report = probe_state(&mut child, Duration::from_secs(3)).unwrap();
    assert_eq!(report.response.data.unwrap()["fixture"], true);
    assert_eq!(report.ignored_records, 2);
    assert_eq!(report.rejected_records, 1);
    assert!(!std::path::Path::new(&format!("/proc/{}", report.pid)).exists());
}

#[test]
fn timeout_reaps_a_silent_child() {
    let mut child = fixture("import time; time.sleep(60)");
    let started = Instant::now();
    assert!(matches!(
        probe_state(&mut child, Duration::from_millis(100)),
        Err(ProbeError::Timeout)
    ));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn eof_and_broken_pipe_are_not_success() {
    let mut child = fixture("import sys; sys.exit(0)");
    assert!(matches!(
        probe_state(&mut child, Duration::from_secs(2)),
        Err(ProbeError::Eof | ProbeError::Write)
    ));
}

#[test]
fn matching_id_cannot_acknowledge_a_different_command() {
    let mut child = fixture(
        r#"
import json,sys
r=json.loads(sys.stdin.readline())
print(json.dumps({'type':'response','id':r['id'],'command':'prompt','success':True}))
"#,
    );
    assert!(matches!(
        probe_state(&mut child, Duration::from_secs(2)),
        Err(ProbeError::MismatchedCommand)
    ));
}

#[test]
fn errors_do_not_expose_backend_diagnostics() {
    let mut child = fixture(
        r#"
import json,sys
r=json.loads(sys.stdin.readline())
print(json.dumps({'type':'response','id':r['id'],'command':'get_state','success':False,'error':'private token'}))
"#,
    );
    let error = probe_state(&mut child, Duration::from_secs(2)).unwrap_err();
    assert!(matches!(error, ProbeError::Rejected));
    assert!(!error.to_string().contains("private"));
}

#[test]
fn zero_timeout_is_rejected_before_spawn() {
    let mut child = Command::new("/does/not/exist");
    assert!(matches!(
        probe_state(&mut child, Duration::ZERO),
        Err(ProbeError::InvalidTimeout)
    ));
}

#[test]
fn output_flood_cannot_extend_the_deadline_or_block_cleanup() {
    let mut child = fixture("while True: print('{\"type\":\"notify\"}')");
    let started = Instant::now();
    assert!(matches!(
        probe_state(&mut child, Duration::from_millis(100)),
        Err(ProbeError::Timeout)
    ));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn inherited_pipes_do_not_delay_timeout_cleanup() {
    let mut child = fixture("import os,time; pid=os.fork(); time.sleep(0.6 if pid == 0 else 60)");
    let started = Instant::now();
    assert!(matches!(
        probe_state(&mut child, Duration::from_millis(50)),
        Err(ProbeError::Timeout)
    ));
    assert!(started.elapsed() < Duration::from_millis(400));
}

#[test]
fn escaped_inherited_pipe_does_not_strand_reader_threads() {
    let mut child = fixture(
        r#"
import os,time
if os.fork() == 0:
    os.setsid()
    time.sleep(0.6)
else:
    time.sleep(60)
"#,
    );
    let started = Instant::now();
    assert!(matches!(
        probe_state(&mut child, Duration::from_millis(50)),
        Err(ProbeError::Timeout)
    ));
    assert!(started.elapsed() < Duration::from_millis(400));
}
