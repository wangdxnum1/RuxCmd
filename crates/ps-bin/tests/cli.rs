use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn shows_help() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn shows_version() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ps"));
}

#[test]
fn runs_default() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"));
}

#[test]
fn custom_columns() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-o", "pid,ppid,comm"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"))
        .stdout(predicate::str::contains("PPID"))
        .stdout(predicate::str::contains("COMM"));
}

#[test]
fn no_headers() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["--no-headers", "-o", "pid,comm"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID").not());
}

#[test]
fn forest_renders() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["--forest", "-e", "-o", "pid,comm"]);
    cmd.assert().success();
}

#[test]
fn unknown_column_warns() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-o", "pid,nonsense"]);
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("unknown column"));
}

#[test]
fn list_columns_works() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--list-columns");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pid"))
        .stdout(predicate::str::contains("comm"));
}
