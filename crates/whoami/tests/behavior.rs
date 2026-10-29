use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_a_nonempty_username() {
    cargo_bin_cmd!("whoami")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

#[test]
fn output_is_a_single_line() {
    let out = cargo_bin_cmd!("whoami").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text.matches('\n').count(), 1);
    assert!(!text.trim().is_empty());
}
