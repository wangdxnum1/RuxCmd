use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
#[test]
fn reads_file_and_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("input with spaces.txt");
    fs::write(&file, b"hello\nworld\n").unwrap();
    cargo_bin_cmd!("cat")
        .arg(&file)
        .assert()
        .success()
        .stdout("hello\nworld\n");
    cargo_bin_cmd!("cat")
        .write_stdin("from stdin\n")
        .assert()
        .success()
        .stdout("from stdin\n");
}
#[test]
fn missing_file_reports_failure() {
    let dir = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("cat")
        .arg(dir.path().join("missing.txt"))
        .assert()
        .code(1)
        .stderr(predicates::str::contains("missing.txt"));
}
