use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Spawn a long-lived child (ping loop) and return its PID.
fn spawn_child() -> u32 {
    let child = Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");
    child.id()
}

/// Kill the child unconditionally (test cleanup), ignoring errors.
fn cleanup(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn pid_alive(pid: u32) -> bool {
    // taskkill /PID with no /F on a dead PID errors; probing via `tasklist`
    // keeps this check dependency-free.
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .expect("tasklist");
    let text = String::from_utf8_lossy(&out.stdout);
    text.contains(&pid.to_string())
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
    let pid = spawn_child();
    assert!(pid_alive(pid), "child must start alive");

    let out = cargo_bin_cmd!("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .unwrap();
    cleanup(pid);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Poll briefly for the process to disappear.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut gone = false;
    while Instant::now() < deadline {
        if !pid_alive(pid) {
            gone = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(gone, "child should terminate after kill");
}

#[test]
fn nonexistent_pid_fails_with_exit_one() {
    // PID 4 is the System process and cannot be terminated by users; use an
    // unused high PID instead.
    cargo_bin_cmd!("kill")
        .args(["-9", "4000000"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("kill:"));
}

#[test]
fn invalid_signal_name_fails() {
    cargo_bin_cmd!("kill")
        .args(["-s", "NOSUCH", "4000000"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid signal"));
}
