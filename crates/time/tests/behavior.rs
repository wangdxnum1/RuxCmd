use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn reports_real_user_sys_lines() {
    let out = cargo_bin_cmd!("time")
        .args(["cmd", "/c", "exit", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("real"), "stdout: {text}");
    assert!(text.contains("user"), "stdout: {text}");
    assert!(text.contains("sys"), "stdout: {text}");
}

#[test]
fn child_exit_code_passes_through() {
    cargo_bin_cmd!("time")
        .args(["cmd", "/c", "exit", "7"])
        .assert()
        .failure()
        .code(7);
}

#[test]
fn gnu_duration_format_used() {
    // GNU time prints `0m0.003s` style durations.
    let out = cargo_bin_cmd!("time")
        .args(["cmd", "/c", "exit", "0"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("m0.") || text.contains("m1.") || text.contains("m2."),
        "expected GNU 0mX.XXXs format, got: {text}"
    );
}

#[test]
fn missing_command_fails() {
    cargo_bin_cmd!("time")
        .arg("definitely-missing-binary.exe")
        .assert()
        .failure()
        .stderr(predicates::str::contains("time:").or(predicates::str::contains("No such")));
}
