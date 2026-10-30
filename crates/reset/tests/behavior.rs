use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn emits_reset_sequence() {
    let out = cargo_bin_cmd!("reset").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    // Home, clear, and reset-scrollback ANSI sequence.
    assert_eq!(out.stdout, b"\x1b[H\x1b[2J\x1b[3J");
}

#[test]
fn version_flag_exits_zero() {
    cargo_bin_cmd!("reset")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("reset"));
}
