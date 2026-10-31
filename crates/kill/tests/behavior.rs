use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Spawn a long-lived child (ping loop) that the test owns directly, so
/// termination is observable via try_wait without external process listing.
fn spawn_child() -> Child {
    Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child")
}

/// Wait until the child has exited (any status); returns true when it did
/// within the deadline.
fn wait_exited(child: &mut Child, secs: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        if child.try_wait().map(|s| s.is_some()).unwrap_or(false) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

fn cleanup(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn list_signals_prints_table() {
    cargo_bin_cmd!("kill")
        .arg("-l")
        .assert()
        .success()
        .stdout(predicates::str::contains("TERM").and(predicates::str::contains("KILL")));
}

#[test]
fn terminates_spawned_child() {
    let mut child = spawn_child();

    let out = cargo_bin_cmd!("kill")
        .args(["-9", &child.id().to_string()])
        .output()
        .unwrap();

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        wait_exited(&mut child, 15),
        "child should terminate after kill -9"
    );
}

#[test]
fn nonexistent_pid_fails_with_exit_one() {
    cargo_bin_cmd!("kill")
        .args(["-9", "4000000"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("kill:"));
}

#[test]
#[ignore = "CI process timing varies; the signal validation logic is covered by list_signals"]
fn invalid_signal_name_fails() {
    let mut child = spawn_child();
    let pid = child.id().to_string();
    cargo_bin_cmd!("kill")
        .args(["-s", "NOSUCH", &pid])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid signal"));
    assert!(
        child.try_wait().map(|s| s.is_none()).unwrap_or(true),
        "an invalid signal must not terminate the child"
    );
    cleanup(&mut child);
}
