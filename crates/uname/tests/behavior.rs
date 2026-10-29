use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn default_output_is_nonempty() {
    cargo_bin_cmd!("uname")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

#[test]
fn kernel_name_is_nonempty() {
    cargo_bin_cmd!("uname")
        .arg("-s")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

#[test]
fn machine_is_an_architecture_token() {
    cargo_bin_cmd!("uname")
        .arg("-m")
        .assert()
        .success()
        .stdout(predicates::str::contains("x86_64").or(predicates::str::contains("AMD64")));
}

#[test]
fn all_option_is_one_line() {
    let out = cargo_bin_cmd!("uname").arg("-a").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text.matches('\n').count(), 1);
    assert!(text.trim().len() > 3, "output: {text:?}");
}

#[test]
fn kernel_release_is_nonempty() {
    cargo_bin_cmd!("uname")
        .arg("-r")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}
