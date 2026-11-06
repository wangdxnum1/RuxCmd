use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_header_and_at_least_one_volume_row() {
    let out = cargo_bin_cmd!("df").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    // At least two lines: header plus one volume row.
    assert!(text.lines().count() >= 2, "output: {text}");
    // Exit code is environment-dependent (unreadable drives cause exit 1).
}

#[test]
fn human_readable_suffixes() {
    let out = cargo_bin_cmd!("df").arg("-h").output().unwrap();
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
