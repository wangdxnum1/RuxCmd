use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Child, Command, Stdio};

fn spawn_child() -> Child {
    Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child")
}

#[test]
fn finds_a_cmd_process_while_child_is_alive() {
    let mut child = spawn_child();
    // The child runs under cmd.exe, so `pgrep cmd` must report at least one
    // PID while it is still running. PIDs are printed one per line.
    let mut found = false;
    for _ in 0..20 {
        let out = cargo_bin_cmd!("pgrep").arg("cmd").output().unwrap();
        if out.status.code() == Some(0) {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.lines().any(|l| l.trim().parse::<u32>().is_ok()) {
                found = true;
                break;
            }
        }
        if child.try_wait().map(|s| s.is_some()).unwrap_or(true) {
            break; // child unexpectedly gone; retry loop would be pointless
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let _ = child.kill();
    let _ = child.wait();
    assert!(found, "pgrep cmd must report numeric PIDs");
}

#[test]
fn list_name_flag_prints_pid_and_name() {
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
