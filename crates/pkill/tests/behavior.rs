use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn pid_alive(pid: u32) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .expect("tasklist");
    String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
}

#[test]
fn kills_child_matching_pattern() {
    let child = Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");
    let pid = child.id();

    // pkill cmd terminates cmd.exe processes; our child must disappear.
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut killed = false;
    while Instant::now() < deadline {
        let out = cargo_bin_cmd!("pkill")
            .args(["-f", "ping"])
            .output()
            .unwrap();
        if !pid_alive(pid) {
            killed = true;
            break;
        }
        let _ = out;
        std::thread::sleep(Duration::from_millis(250));
    }
    // Belt-and-braces cleanup so a failed assertion doesn't leak the child.
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    assert!(killed, "pkill must terminate the ping child");
}

#[test]
fn no_match_exits_one() {
    cargo_bin_cmd!("pkill")
        .arg("definitely-not-a-process-name-xyz")
        .assert()
        .failure()
        .code(1);
}
