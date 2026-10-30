use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_eight_hex_digits() {
    let out = cargo_bin_cmd!("hostid").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(text.len(), 8, "output: {text:?}");
    assert!(
        text.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
}

#[test]
fn stable_across_runs() {
    let a = cargo_bin_cmd!("hostid").output().unwrap();
    let b = cargo_bin_cmd!("hostid").output().unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn changes_with_hostname_environment() {
    // The identifier derives from COMPUTERNAME, so overriding it must move
    // the output (documents the derivation without hardcoding a value).
    let a = cargo_bin_cmd!("hostid").output().unwrap();
    let b = cargo_bin_cmd!("hostid")
        .env("COMPUTERNAME", "other-host-for-testing")
        .output()
        .unwrap();
    assert_ne!(a.stdout, b.stdout);
}
