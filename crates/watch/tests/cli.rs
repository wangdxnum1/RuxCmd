use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

// cargo test puts target/debug first on PATH and this workspace builds its own
// echo.exe/sort.exe there, so resolve cmd.exe by absolute path under
// %SystemRoot% like the nohup tests do.
fn cmd_exe() -> String {
    format!(
        "{}\\System32\\cmd.exe",
        std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())
    )
}

/// -e/--errexit: the loop must end as soon as the command exits non-zero and
/// pass that status through (procps watch exits with the command's status too;
/// it only waits for a key press first, which a headless port cannot do).
/// -x/--exec rides along: it is accepted and changes nothing.
#[test]
fn errexit_exits_with_command_status_quickly() {
    let start = Instant::now();
    cargo_bin_cmd!("watch")
        .args([
            "-n",
            "0.1",
            "-x",
            "-e",
            cmd_exe().as_str(),
            "/c",
            "exit",
            "3",
        ])
        .assert()
        .failure()
        .code(3);
    assert!(
        start.elapsed() < Duration::from_secs(3),
        "errexit did not return promptly: {:?}",
        start.elapsed()
    );
}

/// The header line shows the interval, the command text and the local clock;
/// the command's output appears below the rule. -e with a failing command
/// ends the loop after the first iteration so the whole run stays short. The
/// command is kept short enough to fit on the 80-column header uncropped.
#[test]
fn prints_title_rule_and_command_output() {
    cargo_bin_cmd!("watch")
        .args([
            "-n",
            "0.1",
            "-e",
            cmd_exe().as_str(),
            "/c",
            "echo hi& exit 5",
        ])
        .assert()
        .failure()
        .code(5)
        .stdout(predicate::str::contains("Every 0.1s"))
        .stdout(predicate::str::contains("echo hi& exit 5"))
        .stdout(predicate::str::contains("hi\n"));
}

/// --no-title must drop the header and the rule while keeping the output.
#[test]
fn no_title_suppresses_header_and_rule() {
    cargo_bin_cmd!("watch")
        .args([
            "-n",
            "0.1",
            "--no-title",
            "-e",
            cmd_exe().as_str(),
            "/c",
            "echo hi& exit 4",
        ])
        .assert()
        .failure()
        .code(4)
        .stdout(predicate::str::contains("hi\n"))
        .stdout(predicate::str::contains("Every").not());
}

/// Spawns `watch -n 0.2 [extra] cmd /c type w.txt` in `dir`, whose w.txt the
/// caller rewrites mid-run. cmd /c keeps the script free of quoting hazards.
fn spawn_type_watch(extra: &[&str], dir: &std::path::Path) -> std::process::Child {
    let mut args: Vec<&str> = vec!["-n", "0.2"];
    args.extend_from_slice(extra);
    let cmd = cmd_exe();
    args.extend_from_slice(&[cmd.as_str(), "/c", "type w.txt"]);
    Command::new(env!("CARGO_BIN_EXE_watch"))
        .args(&args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn watch")
}

/// Waits up to 3s for the child to exit; kills it and fails otherwise.
fn wait_limited(child: &mut std::process::Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait().expect("try_wait watch") {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("watch did not exit within 3s");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// -g/--chgexit: the first run is the baseline; when a later run's output
/// differs from the previous one, watch must exit (status 0) promptly.
#[test]
fn chgexit_exits_when_output_changes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("w.txt"), "A\n").unwrap();
    let mut child = spawn_type_watch(&["-g"], dir.path());
    // Let at least two iterations observe the unchanged file first.
    std::thread::sleep(Duration::from_millis(600));
    std::fs::write(dir.path().join("w.txt"), "B\n").unwrap();
    let status = wait_limited(&mut child);
    assert_eq!(status.code(), Some(0));
}

/// -d/--differences: lines that changed since the previous run must be
/// wrapped in reverse video (the first run is the baseline and is never
/// highlighted), and the feature must not panic.
#[test]
fn differences_highlight_changed_lines() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("w.txt"), "A\n").unwrap();
    let mut child = spawn_type_watch(&["-d", "-g"], dir.path());
    std::thread::sleep(Duration::from_millis(600));
    std::fs::write(dir.path().join("w.txt"), "B\n").unwrap();
    let status = wait_limited(&mut child);
    assert_eq!(status.code(), Some(0));
    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    assert!(
        out.contains("\x1B[7mB\x1B[0m"),
        "changed line not highlighted in reverse video: {out:?}"
    );
}
