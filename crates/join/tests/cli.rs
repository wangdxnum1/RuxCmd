use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn write_pair(dir: &TempDir) -> (String, String) {
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    fs::write(&a, "1 x\n2 y\n").unwrap();
    fs::write(&b, "1 p\n2 q\n").unwrap();
    (a.to_string_lossy().into_owned(), b.to_string_lossy().into_owned())
}

#[test]
fn zero_field_number_for_file1_is_rejected() {
    // Regression: `join -1 0` used to underflow `args.field1 - 1` (usize).
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_pair(&dir);
    cargo_bin_cmd!("join")
        .args(["-1", "0", "-2", "1", &a, &b])
        .assert()
        .failure()
        .code(1)
        .stderr("join: invalid field number '0'\n")
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn zero_field_number_for_file2_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_pair(&dir);
    cargo_bin_cmd!("join")
        .args(["-1", "1", "-2", "0", &a, &b])
        .assert()
        .failure()
        .code(1)
        .stderr("join: invalid field number '0'\n")
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn join_on_first_field_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let (a, b) = write_pair(&dir);
    cargo_bin_cmd!("join")
        .args(["-t", " ", &a, &b])
        .assert()
        .success()
        .stdout(predicates::str::contains("1 x"))
        .stdout(predicates::str::contains("2 y"));
}
