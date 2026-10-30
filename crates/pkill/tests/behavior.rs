use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn kills_child_matching_pattern() {
    let mut child = Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1", ">nul"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");

    // pkill ping terminates ping-bearing processes; our own child must exit
    // (observed through the owned handle, no external process listing).
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut killed = false;
    while Instant::now() < deadline {
        let _ = cargo_bin_cmd!("pkill").arg("ping").output().unwrap();
        if child.try_wait().map(|s| s.is_some()).unwrap_or(false) {
            killed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = child.kill();
    let _ = child.wait();
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
