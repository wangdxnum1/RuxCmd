//! Integration tests for env: NAME=VALUE assignments with command execution,
//! print-mode with -u, and -i with assignments.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_env")
}

fn system_cmd() -> PathBuf {
    let mut p = PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    p.push("System32");
    p.push("cmd.exe");
    p
}

#[test]
fn assignments_reach_child_command() {
    let out = Command::new(bin())
        .arg("FOO=bar")
        .arg(system_cmd())
        .args(["/c", "set", "FOO"])
        .output()
        .unwrap();
    assert!(out.status.success(), "code: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FOO=bar"), "stdout: {:?}", stdout);
}

#[test]
fn print_mode_honors_unset() {
    let out = Command::new(bin()).args(["-u", "PATH"]).output().unwrap();
    assert!(out.status.success(), "code: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Windows variable names are case-insensitive; compare accordingly.
    let is_path = |l: &str| l.to_ascii_uppercase().starts_with("PATH=");
    assert!(
        !stdout.lines().any(is_path),
        "PATH should be filtered: {:?}",
        stdout
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.to_ascii_uppercase().starts_with("SYSTEMROOT=")),
        "other variables should survive: {:?}",
        stdout
    );
}

#[test]
fn ignore_environment_prints_only_assignments() {
    let out = Command::new(bin())
        .args(["-i", "FOO=bar"])
        .output()
        .unwrap();
    assert!(out.status.success(), "code: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim_end(), "FOO=bar", "stdout: {:?}", stdout);
}
