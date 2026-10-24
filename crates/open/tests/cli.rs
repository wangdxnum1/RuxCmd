//! Tests for the P1 fixes: open must go through ShellExecuteW (no
//! cmd/powershell quoting/injection), -R must succeed despite explorer's
//! exit code 1, and -W must wait for the launched process to exit.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::time::Instant;

#[test]
fn opens_local_file_with_default_association() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.txt");
    fs::write(&file, "hello").unwrap();
    // ShellExecuteW launches the associated app asynchronously; the command
    // itself must report success immediately (a notepad window may appear).
    cargo_bin_cmd!("open").arg(&file).assert().success();
}

#[test]
fn application_mode_runs_program_with_arguments() {
    // cmd.exe /c exit terminates immediately: exercises ShellExecuteW with
    // lpParameters and leaves no process behind.
    cargo_bin_cmd!("open")
        .args(["-a", "cmd.exe", "--", "/c", "exit"])
        .assert()
        .success();
}

#[test]
fn wait_blocks_until_application_exits() {
    // ping -n 2 takes about one second; -W must not return before the
    // application exits.
    let start = Instant::now();
    cargo_bin_cmd!("open")
        .args(["-W", "-a", "ping", "--", "-n", "2", "127.0.0.1"])
        .assert()
        .success();
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 800,
        "-W returned after {elapsed:?}; it did not wait for the application"
    );
}

#[test]
fn reveal_file_in_explorer_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    fs::write(&file, "x").unwrap();
    // explorer.exe /select routinely exits with code 1 on success; open
    // must report exit 0 for both files and directories.
    cargo_bin_cmd!("open")
        .arg("-R")
        .arg(&file)
        .assert()
        .success();
    cargo_bin_cmd!("open")
        .arg("-R")
        .arg(dir.path())
        .assert()
        .success();
}

#[test]
#[ignore] // smoke test: opens the default web browser
fn url_containing_ampersand_is_passed_through() {
    // The URL must reach the browser intact (& is a cmd metacharacter; with
    // ShellExecuteW there is no cmd shell to truncate it).
    cargo_bin_cmd!("open")
        .arg("https://x.com/?a=1&b=2")
        .assert()
        .success();
}
