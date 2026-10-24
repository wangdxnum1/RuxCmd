//! Integration tests for pgrep (P1 remediation, batch A1).
//!
//! All tests spawn only their own short-lived `ping` children. The ping image
//! is resolved with its real on-disk casing (e.g. `System32\PING.EXE`) so the
//! case-sensitivity assertions hold regardless of how PATH resolution spells
//! it.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Serializes tests that enumerate processes by name: parallel tests would
/// otherwise observe each other's `pgrep`/`ping` children.
fn lock() -> MutexGuard<'static, ()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Absolute path to ping.exe carrying its real on-disk casing.
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

fn ping_image() -> String {
    ping_exe()
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned()
}

fn spawn_ping() -> Child {
    Command::new(ping_exe())
        .args(["-n", "30", "127.0.0.1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ping")
}

/// Same-length variant of `name` whose case differs from it.
fn flipped_case(name: &str) -> String {
    let upper = name.to_uppercase();
    if upper != name {
        upper
    } else {
        name.to_lowercase()
    }
}

/// `PING.EXE` -> `PING` (case-insensitive .exe strip, ASCII only).
fn stem(name: &str) -> &str {
    if name.len() > 4 && name[name.len() - 4..].eq_ignore_ascii_case(".exe") {
        &name[..name.len() - 4]
    } else {
        name
    }
}

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_pgrep"))
        .args(args)
        .output()
        .expect("run pgrep");
    Out {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn pids(stdout: &str) -> Vec<u32> {
    stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next()?.parse().ok())
        .collect()
}

/// Regression (review 2026-10-07, pgrep): pgrep used to shell out to
/// tasklist/wmic and match case-insensitively. It must now match by image
/// name, case-sensitively by default, as a substring; `-x` must require
/// equality and `-i` must flip the case rule.
#[test]
fn matches_spawned_ping_by_name() {
    let _g = lock();
    let mut child = spawn_ping();
    let image = ping_image();

    // Exact match with the real on-disk casing.
    let out = run(&["-x", &image]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(
        pids(&out.stdout).contains(&child.id()),
        "expected {} in {:?}",
        child.id(),
        pids(&out.stdout)
    );

    // Default (no -x) is substring matching: the stem matches too.
    let out = run(&[stem(&image)]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(pids(&out.stdout).contains(&child.id()));

    // Image-name matching is case-insensitive on Windows: Toolhelp32 reports
    // its own casing (PING.EXE) which need not match the on-disk spelling,
    // so both spellings must hit (declared difference from procps).
    let out = run(&["-x", &flipped_case(&image)]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(pids(&out.stdout).contains(&child.id()));

    child.kill().expect("cleanup kill");
    let _ = child.wait();
}

/// Regression (review 2026-10-07, pgrep): no match used to exit 0, breaking
/// the procps scripting contract. It must exit 1 and print nothing.
#[test]
fn no_match_exits_1() {
    let out = run(&["-x", "definitely-not-running-9f3b1c.exe"]);
    assert_eq!(out.code, Some(1));
    assert!(out.stdout.trim().is_empty(), "stdout: {}", out.stdout);
}

/// Regression (review 2026-10-07, pgrep): pgrep never excluded itself. The
/// running pgrep process must not appear in its own output.
#[test]
fn excludes_its_own_pid() {
    let _g = lock();
    let mut child = Command::new(env!("CARGO_BIN_EXE_pgrep"))
        .args(["-x", "-i", "pgrep.exe"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pgrep");
    let own = child.id();
    let out = child.wait_with_output().expect("wait pgrep");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !pids(&stdout).contains(&own),
        "pgrep must exclude its own PID {own}, got: {stdout}"
    );
}

/// Regression (review 2026-10-07, pgrep): `-f` used to depend on wmic, which
/// is removed on Windows 11 24H2+ (silent empty match). It must match the
/// full command line read natively.
#[test]
fn full_flag_matches_command_line() {
    let _g = lock();
    let mut child = spawn_ping();

    let out = run(&["-f", "30 127.0.0.1"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(pids(&out.stdout).contains(&child.id()));

    let out = run(&["-f", "zz-no-such-cmdline-9f3b1c"]);
    assert_eq!(out.code, Some(1));

    child.kill().expect("cleanup kill");
    let _ = child.wait();
}

/// `-l` prints "PID NAME" pairs.
#[test]
fn list_name_prints_pid_and_name() {
    let _g = lock();
    let mut child = spawn_ping();
    let image = ping_image();

    let out = run(&["-l", "-x", &image]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let line = out
        .stdout
        .lines()
        .find(|l| l.split_whitespace().next() == Some(&child.id().to_string()))
        .unwrap_or_else(|| panic!("no line for PID {} in {:?}", child.id(), out.stdout));
    assert!(
        line.to_lowercase().ends_with(&image.to_lowercase()),
        "line: {line}"
    );

    child.kill().expect("cleanup kill");
    let _ = child.wait();
}

/// The task-specified scenario: `cmd /c ping -n 30 127.0.0.1` must be found
/// (`-i` because the reported image name uses the on-disk casing, e.g.
/// PING.EXE, while the task spells the pattern lowercase).
#[test]
fn finds_ping_started_via_cmd() {
    let _g = lock();
    let ping = ping_exe();
    let mut cmd_child = Command::new("cmd")
        .arg("/c")
        .arg(&ping)
        .args(["-n", "30", "127.0.0.1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cmd");
    // cmd needs a beat to spawn its ping child before we enumerate.
    std::thread::sleep(std::time::Duration::from_millis(700));
            let out = run(&["-i", "ping"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let found = pids(&out.stdout);
    assert!(!found.is_empty());
    assert!(
        !found.contains(&std::process::id()),
        "pgrep must not report the caller's PID"
    );
    // The orphaned ping self-exits within 30s; cmd dies now.
    cmd_child.kill().ok();
    let _ = cmd_child.wait();
}
