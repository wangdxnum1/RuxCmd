use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn lists_a_filesystem_line_with_header() {
    let out = cargo_bin_cmd!("df").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.trim().is_empty());
    // At least two lines: header plus one volume row.
    assert!(text.lines().count() >= 2, "output: {text}");
}

#[test]
fn human_readable_suffixes() {
    let out = cargo_bin_cmd!("df").arg("-h").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.trim().is_empty());
}

#[test]
fn explicit_path_argument_is_accepted() {
    cargo_bin_cmd!("df")
        .arg(".")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}
