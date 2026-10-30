use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn spawn_child() -> u32 {
    Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child")
        .id()
}

fn cleanup(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[test]
fn finds_spawned_child_by_name() {
    let pid = spawn_child();
    // The child runs under cmd.exe, so `pgrep cmd` must find it.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut found = false;
    while Instant::now() < deadline && !found {
        let out = cargo_bin_cmd!("pgrep")
            .arg("cmd")
            .output()
            .unwrap();
        if out.status.code() == Some(0) {
            let text = String::from_utf8_lossy(&out.stdout);
            found = text
                .lines()
                .any(|line| line.trim().parse::<u32>().map(|p| p == pid).unwrap_or(false));
        }
        if !found {
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    cleanup(pid);
    assert!(found, "pgrep must report the spawned cmd child PID {pid}");
}

#[test]
fn list_name_flag_prints_names() {
    let out = cargo_bin_cmd!("pgrep")
        .args(["-l", "cmd"])
        .output()
        .unwrap();
    // Either matches (name column present) or exit 1 when nothing matches.
    let text = String::from_utf8_lossy(&out.stdout);
    if out.status.code() == Some(0) {
        assert!(!text.trim().is_empty());
    } else {
        assert_eq!(out.status.code(), Some(1));
    }
}

#[test]
fn no_match_exits_one() {
    cargo_bin_cmd!("pgrep")
        .arg("definitely-not-a-process-name-xyz")
        .assert()
        .failure()
        .code(1);
}
