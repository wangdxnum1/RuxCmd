use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn message_with_tag_goes_to_stderr() {
    cargo_bin_cmd!("logger")
        .args(["-s", "-t", "mytag", "hello"])
        .assert()
        .success()
        .stderr(predicates::str::contains("mytag: hello"));
}

#[test]
fn reads_stdin_when_no_message() {
    cargo_bin_cmd!("logger")
        .args(["-s"])
        .write_stdin("line1\nline2\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("line1").and(predicates::str::contains("line2")));
}

#[test]
fn no_args_reads_empty_stdin() {
    cargo_bin_cmd!("logger").assert().success();
}

#[test]
fn tag_defaults_to_username() {
    cargo_bin_cmd!("logger")
        .args(["-s", "msg"])
        .assert()
        .success()
        .stderr(predicates::str::contains(": msg"));
}
