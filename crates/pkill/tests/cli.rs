//! Integration tests for pkill (P1 remediation, batch A1).
//!
//! All tests spawn only their own short-lived `ping` children and terminate
//! them before returning (or they self-exit via `ping -n 30`).

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

/// Serializes tests that enumerate processes by name: parallel tests would
/// otherwise observe each other's `pkill`/`ping` children.
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

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_pkill"))
        .args(args)
        .output()
        .expect("run pkill");
    Out {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Waits for `child` and asserts it died within `limit`.
fn assert_dies_quickly(mut child: Child, limit: Duration) {
    let start = Instant::now();
    let status = child.wait().expect("wait child");
    assert!(
        start.elapsed() < limit,
        "child did not terminate promptly ({:?})",
        start.elapsed()
    );
    assert!(!status.success(), "child exited cleanly; expected a kill");
}

/// Regression (review 2026-10-07, pkill): pkill used to shell out to
/// tasklist/taskkill and count a spawned-but-failed taskkill as success.
/// `pkill -x <image>` must now really terminate the matched process and
/// exit 0.
#[test]
fn exact_name_kill_terminates_spawned_process() {
    let _g = lock();
    let child = spawn_ping();
    let out = run(&["-x", &ping_image()]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert_dies_quickly(child, Duration::from_secs(5));
}

/// Image-name matching is ASCII case-insensitive on Windows (Toolhelp32
/// casing is unreliable, same declared behavior as pgrep): a case-flipped
/// `-x` pattern still matches, with or without -i.
#[test]
fn case_insensitive_default_and_ignore_case_flag() {
    let _g = lock();
    let mut child = spawn_ping();
    let flipped = flipped_case(&ping_image());

    let out = run(&["-x", &flipped]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert_dies_quickly(child, Duration::from_secs(5));

    let mut child = spawn_ping();
    let out = run(&["-x", "-i", &flipped]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert_dies_quickly(child, Duration::from_secs(5));
}

/// Regression (review 2026-10-07, pkill): no match used to exit 0 (because a
/// spawned taskkill counted as killed). No match must exit 1.
#[test]
fn no_match_exits_1() {
    let out = run(&["-x", "no-such-image-9f3b1c.exe"]);
    assert_eq!(out.code, Some(1));
}

/// Regression (review 2026-10-07, pkill): pkill never excluded itself, so
/// `pkill pkill` killed its own process. `pkill -x pkill.exe` with no other
/// instance must find nothing and survive to report exit 1.
#[test]
fn excludes_its_own_pid() {
    let _g = lock();
    let mut child = Command::new(env!("CARGO_BIN_EXE_pkill"))
        .args(["-x", "pkill.exe"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pkill");
    let out = child.wait_with_output().expect("wait pkill");
    assert_eq!(
        out.status.code(),
        Some(1),
        "pkill -x pkill.exe must match nothing (self excluded) and exit 1"
    );
}

/// Regression (review 2026-10-07, pkill): `-f` used to depend on wmic,
/// removed on Windows 11 24H2+. It must match the full command line read
/// natively and terminate the match.
#[test]
fn full_flag_matches_command_line() {
    let _g = lock();
    let child = spawn_ping();

    let out = run(&["-f", "30 127.0.0.1"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert_dies_quickly(child, Duration::from_secs(5));
}
