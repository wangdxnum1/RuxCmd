use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Run the timeout binary with null stdio and return (exit code, elapsed).
///
/// The null stdio is deliberate: assert_cmd always pipes stdout, and a
/// `cmd /c <grandchild>` inherits that pipe write end (CreateProcess with
/// handle inheritance), so the harness would keep waiting for pipe EOF until
/// the orphaned grandchild exits even though timeout itself exited 124 long
/// before. Waiting on the process handle measures timeout's own exit.
fn run_with_null_stdio(args: &[&str]) -> (Option<i32>, Duration) {
    let start = Instant::now();
    let status = Command::new(env!("CARGO_BIN_EXE_timeout"))
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run timeout");
    (status.code(), start.elapsed())
}

fn assert_code_and_window(code: Option<i32>, elapsed: Duration, expected: i32, context: &str) {
    assert_eq!(code, Some(expected), "{}", context);
    assert!(
        elapsed >= Duration::from_millis(900) && elapsed < Duration::from_secs(4),
        "{}: expected ~1s wait before {}, got {:?}",
        context,
        expected,
        elapsed
    );
}

/// Regression (P0 #37, defect 1 + 2): a child that exits early used to keep
/// timeout blocked for the whole duration (killer thread slept for `duration`
/// before taskkill, and `wait()` was unconditionally joined afterwards), and
/// exit code 124 was never produced because `status.code().is_none()` is always
/// false on Windows.
///
/// After the fix `timeout 3600s cmd /c exit 7` must return the child's own
/// exit code 7 within ~3 seconds instead of hanging for an hour.
#[test]
fn early_exit_returns_child_code_promptly() {
    let start = Instant::now();
    cargo_bin_cmd!("timeout")
        .args(["3600s", "cmd", "/c", "exit", "7"])
        .assert()
        .code(7);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "timeout blocked on an already-exited child for {:?}",
        elapsed
    );
}

/// Regression (P0 #37, defect 2): an interval that expires while the child is
/// still running must kill it and exit 124 at about the deadline. The old code
/// taskkill'ed the child and then exited with the child's termination status
/// (1 on Windows), never 124.
///
/// `cmd /c ping -n 10 127.0.0.1` runs for ~9s, so a 1s timeout must return 124
/// within a 1-4s window.
#[test]
fn expiry_kills_child_and_returns_124() {
    let (code, elapsed) =
        run_with_null_stdio(&["1s", "cmd", "/c", "ping", "-n", "10", "127.0.0.1"]);
    assert_code_and_window(code, elapsed, 124, "expiry with cmd wrapper");
}

/// Same scenario with the long-running process as the direct child (no cmd
/// wrapper): the kill must land on the child itself.
#[test]
fn expiry_kills_direct_child_and_returns_124() {
    let (code, elapsed) = run_with_null_stdio(&["1s", "ping", "-n", "10", "127.0.0.1"]);
    assert_code_and_window(code, elapsed, 124, "expiry with direct child");
}

/// Regression (P0 #37, defect 4): an unparseable interval used to fall back to
/// `unwrap_or(0)` and kill the child immediately. It must be rejected with
/// `timeout: invalid time interval '1x'` and exit 1, without running the
/// command to completion under a 0s timer.
#[test]
fn invalid_interval_errors_instead_of_zero() {
    cargo_bin_cmd!("timeout")
        .args(["1x", "cmd", "/c", "exit", "0"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "timeout: invalid time interval '1x'",
        ));
}

/// Regression (P0 #37, defect 4): fractional durations such as `1.5s` used to
/// fail u64 parsing and collapse to 0, racing an immediate kill. They must be
/// accepted and the child allowed to finish normally.
#[test]
fn fractional_seconds_accepted() {
    let start = Instant::now();
    cargo_bin_cmd!("timeout")
        .args(["1.5s", "cmd", "/c", "exit", "0"])
        .assert()
        .code(0);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "fractional duration rejected or delayed: {:?}",
        elapsed
    );
}

/// No-suffix durations default to seconds.
#[test]
fn plain_duration_defaults_to_seconds() {
    cargo_bin_cmd!("timeout")
        .args(["2", "cmd", "/c", "exit", "5"])
        .assert()
        .code(5);
}

/// `-k/--kill-after` was declared but never read (P0 #37, defect 3). It must
/// now be honored: kill at the deadline, allow kill_after more seconds for the
/// child to die, kill again if it survives, and still exit 124.
#[test]
fn kill_after_is_honored() {
    let (code, elapsed) = run_with_null_stdio(&[
        "-k",
        "1",
        "1s",
        "cmd",
        "/c",
        "ping",
        "-n",
        "10",
        "127.0.0.1",
    ]);
    assert_code_and_window(code, elapsed, 124, "expiry with -k");
}

#[test]
fn kill_after_long_form_is_honored() {
    let (code, elapsed) = run_with_null_stdio(&[
        "--kill-after=1",
        "1s",
        "cmd",
        "/c",
        "ping",
        "-n",
        "10",
        "127.0.0.1",
    ]);
    assert_code_and_window(code, elapsed, 124, "expiry with --kill-after");
}

/// An invalid kill-after interval must be rejected too.
#[test]
fn kill_after_invalid_interval_errors() {
    cargo_bin_cmd!("timeout")
        .args(["-k", "1x", "1s", "cmd", "/c", "exit", "0"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "timeout: invalid time interval '1x'",
        ));
}
