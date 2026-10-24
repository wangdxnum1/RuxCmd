//! Integration tests for kill (P1 remediation, batch A1).
//!
//! All tests spawn only their own short-lived children (`ping` from System32
//! and `cmd /c exit`), which are terminated before returning.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

fn ping_exe() -> PathBuf {
    let sysroot = std::env::var_os("SystemRoot").expect("SystemRoot");
    let dir = PathBuf::from(&sysroot).join("System32");
    let entry = std::fs::read_dir(&dir)
        .expect("read System32")
        .map(|e| e.expect("dir entry"))
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("ping.exe")
        })
        .unwrap_or_else(|| panic!("ping.exe not found in {}", dir.display()));
    entry.path()
}

fn spawn_ping() -> Child {
    Command::new(ping_exe())
        .args(["-n", "30", "127.0.0.1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ping")
}

/// Spawns `cmd /c exit 0`, waits for it, and returns its (now free) PID.
fn exited_pid() -> u32 {
    let mut child = Command::new("cmd")
        .args(["/c", "exit", "0"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cmd");
    let _ = child.wait().expect("wait cmd");
    child.id()
}

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_kill"))
        .args(args)
        .output()
        .expect("run kill");
    Out {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Regression (review 2026-10-07, kill): failures always exited 0. Killing a
/// nonexistent PID must print an error on stderr and exit 1.
#[test]
fn nonexistent_pid_exits_1_with_error() {
    let pid = exited_pid();
    let out = run(&[&pid.to_string()]);
    assert_eq!(out.code, Some(1));
    assert!(
        out.stderr.contains("kill:") && !out.stderr.trim().is_empty(),
        "stderr: {}",
        out.stderr
    );
}

/// Regression (review 2026-10-07, kill): `kill -9 PID` (attached signal
/// form) used to be rejected by clap as an unknown flag. All spellings of a
/// valid signal must work and terminate the process.
#[test]
fn attached_and_named_signal_forms_kill_process() {
    for variant in [
        vec!["-9"],
        vec!["-15"],
        vec!["-KILL"],
        vec!["-TERM"],
        vec!["-SIGKILL"],
        vec!["-s", "KILL"],
        vec!["-s", "kill"],
        vec!["-s", "sigterm"],
        vec!["-s", "9"],
    ] {
        let mut child = spawn_ping();
        let mut args = variant.clone();
        let pid = child.id().to_string();
        args.push(pid.as_str());
        let out = run(&args);
        assert_eq!(
            out.code,
            Some(0),
            "variant {variant:?}: stderr: {}",
            out.stderr
        );
        let status = child.wait().expect("wait ping");
        assert!(!status.success(), "variant {variant:?}: child survived");
    }
}

/// Regression (review 2026-10-07, kill): the signal string was never
/// validated; a typo silently meant TERM. An unknown signal must print
/// `kill: <x>: invalid signal` and exit 1 without touching the PID.
#[test]
fn invalid_signal_is_rejected() {
    let mut child = spawn_ping();
    let pid = child.id().to_string();
    for bad in ["XYZ", "TRM", "99", "SIGZILCH"] {
        let out = run(&["-s", bad, pid.as_str()]);
        assert_eq!(out.code, Some(1), "-s {bad}");
        assert!(
            out.stderr.contains(&format!("kill: {bad}: invalid signal")),
            "-s {bad}: stderr: {}",
            out.stderr
        );
        let out = run(&[&format!("-{bad}"), pid.as_str()]);
        assert_eq!(out.code, Some(1), "-{bad}");
        assert!(
            out.stderr.contains(&format!("kill: {bad}: invalid signal")),
            "-{bad}: stderr: {}",
            out.stderr
        );
    }
    assert!(
        child.try_wait().expect("try_wait").is_none(),
        "child must survive invalid-signal attempts"
    );
    child.kill().expect("cleanup kill");
    let _ = child.wait();
}

/// Regression (review 2026-10-07, kill): `-l` listed only TERM/KILL. It must
/// list the full supported table.
#[test]
fn list_signals_shows_full_table() {
    let out = run(&["-l"]);
    assert_eq!(out.code, Some(0));
    for name in ["HUP", "INT", "QUIT", "ABRT", "KILL", "TERM"] {
        assert!(
            out.stdout.contains(name),
            "missing {name} in {}",
            out.stdout
        );
    }
}

/// A run with one bad and one good PID must still kill the good one, report
/// the bad one, and exit 1.
#[test]
fn failure_aggregation_exits_1_but_kills_rest() {
    let dead = exited_pid();
    let mut child = spawn_ping();
    let out = run(&["-9", &dead.to_string(), &child.id().to_string()]);
    assert_eq!(out.code, Some(1), "stderr: {}", out.stderr);
    let status = child.wait().expect("wait ping");
    assert!(!status.success(), "live PID must still be killed");
}

/// `kill -0 PID` is the existence probe: success without terminating.
#[test]
fn signal_zero_probes_without_terminating() {
    let mut child = spawn_ping();
    let out = run(&["-0", &child.id().to_string()]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(
        child.try_wait().expect("try_wait").is_none(),
        "child must survive kill -0"
    );
    child.kill().expect("cleanup kill");
    let _ = child.wait();
}
