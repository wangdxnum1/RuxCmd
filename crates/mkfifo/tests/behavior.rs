use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn valid_name_exits_zero() {
    let dir = TempDir::new().unwrap();
    cargo_bin_cmd!("mkfifo")
        .arg(dir.path().join("fifo1"))
        .assert()
        .success();
}

#[test]
fn transient_pipe_can_be_recreated() {
    // The server handle is closed immediately, so the name is free again;
    // this locks the documented transient Windows behavior.
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("fifo2");
    cargo_bin_cmd!("mkfifo").arg(&p).assert().success();
    cargo_bin_cmd!("mkfifo").arg(&p).assert().success();
}

#[test]
fn missing_operand_is_an_error() {
    cargo_bin_cmd!("mkfifo")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("mkfifo: missing operand"));
}

#[test]
fn path_without_filename_is_rejected() {
    // A drive root has no file name component for the pipe name.
    cargo_bin_cmd!("mkfifo")
        .arg("C:\\")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("mkfifo:"));
}
