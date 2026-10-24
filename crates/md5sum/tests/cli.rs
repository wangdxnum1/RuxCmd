use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const MD5_HELLO: &str = "5d41402abc4b2a76b9719d911017c592";
const MD5_EMPTY: &str = "d41d8cd98f00b204e9800998ecf8427e";

#[test]
fn check_accepts_binary_star_marker() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.bin"), b"hello").unwrap();
    // GNU md5sum -b output: "<hash> *<file>"
    fs::write(
        dir.path().join("chk"),
        format!("{MD5_HELLO} *data.bin\n"),
    )
    .unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["-c", "chk"])
        .assert()
        .success()
        .stdout("data.bin: OK\n");
}

#[test]
fn check_strips_utf8_bom() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    // Notepad-style UTF-8 BOM before the digest.
    fs::write(
        dir.path().join("chk"),
        format!("\u{feff}{MD5_HELLO} f.txt\n"),
    )
    .unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["-c", "chk"])
        .assert()
        .success()
        .stdout("f.txt: OK\n");
}

#[test]
fn check_processes_all_files_before_exiting_nonzero() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f1.txt"), b"hello").unwrap();
    // First check file: one bad entry. Second check file: valid.
    fs::write(
        dir.path().join("c1.txt"),
        format!("{} f1.txt\n", "0".repeat(32)),
    )
    .unwrap();
    fs::write(dir.path().join("c2.txt"), format!("{MD5_HELLO} f1.txt\n")).unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["-c", "c1.txt", "c2.txt"])
        .assert()
        .code(1)
        .stdout("f1.txt: OK\n")
        .stderr("f1.txt: FAILED\nmd5sum: WARNING: 1 computed checksum did NOT match\n");
}

#[test]
fn tag_format_for_file_and_stdin() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["--tag", "f.txt"])
        .assert()
        .success()
        .stdout(format!("MD5 (f.txt) = {MD5_HELLO}\n"));
    cargo_bin_cmd!("md5sum")
        .args(["--tag"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(format!("MD5 (-) = {MD5_EMPTY}\n"));
}

#[test]
fn status_is_silent_and_reports_exit_code_only() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    fs::write(dir.path().join("bad"), format!("{} f.txt\n", "0".repeat(32))).unwrap();
    fs::write(dir.path().join("good"), format!("{MD5_HELLO} f.txt\n")).unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["--status", "-c", "bad"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["--status", "-c", "good"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn tag_with_check_is_rejected() {
    cargo_bin_cmd!("md5sum")
        .args(["--tag", "-c"])
        .write_stdin("")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("meaningless"));
}

#[test]
fn check_file_without_valid_lines_is_an_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("chk"), "not-a-checksum-line\n").unwrap();
    cargo_bin_cmd!("md5sum")
        .current_dir(dir.path())
        .args(["-c", "chk"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "no properly formatted MD5 checksum lines found",
        ));
}
