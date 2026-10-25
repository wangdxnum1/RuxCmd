use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const SHA_HELLO: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
const SHA_EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[test]
fn check_accepts_binary_star_marker() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.bin"), b"hello").unwrap();
    fs::write(dir.path().join("chk"), format!("{SHA_HELLO} *data.bin\n")).unwrap();
    cargo_bin_cmd!("sha256sum")
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
    fs::write(
        dir.path().join("chk"),
        format!("\u{feff}{SHA_HELLO} f.txt\n"),
    )
    .unwrap();
    cargo_bin_cmd!("sha256sum")
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
    fs::write(dir.path().join("f2.txt"), b"world").unwrap();
    fs::write(
        dir.path().join("c1.txt"),
        format!("{} f1.txt\n", "0".repeat(64)),
    )
    .unwrap();
    fs::write(dir.path().join("c2.txt"), format!("{SHA_HELLO} f1.txt\n")).unwrap();
    cargo_bin_cmd!("sha256sum")
        .current_dir(dir.path())
        .args(["-c", "c1.txt", "c2.txt"])
        .assert()
        .code(1)
        .stdout("f1.txt: OK\n")
        .stderr("f1.txt: FAILED\nsha256sum: WARNING: 1 computed checksum did NOT match\n");
}

#[test]
fn tag_format_for_file_and_stdin() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    cargo_bin_cmd!("sha256sum")
        .current_dir(dir.path())
        .args(["--tag", "f.txt"])
        .assert()
        .success()
        .stdout(format!("SHA256 (f.txt) = {SHA_HELLO}\n"));
    cargo_bin_cmd!("sha256sum")
        .args(["--tag"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(format!("SHA256 (-) = {SHA_EMPTY}\n"));
}

#[test]
fn supports_dash_stdin() {
    cargo_bin_cmd!("sha256sum")
        .arg("-")
        .write_stdin("hello")
        .assert()
        .success()
        .stdout(format!("{SHA_HELLO}  -\n"));
}

#[test]
fn status_is_silent_and_reports_exit_code_only() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    fs::write(
        dir.path().join("bad"),
        format!("{} f.txt\n", "0".repeat(64)),
    )
    .unwrap();
    fs::write(dir.path().join("good"), format!("{SHA_HELLO} f.txt\n")).unwrap();
    cargo_bin_cmd!("sha256sum")
        .current_dir(dir.path())
        .args(["--status", "-c", "bad"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
    cargo_bin_cmd!("sha256sum")
        .current_dir(dir.path())
        .args(["--status", "-c", "good"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn check_file_without_valid_lines_is_an_error() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("chk"), "not-a-checksum-line\n").unwrap();
    cargo_bin_cmd!("sha256sum")
        .current_dir(dir.path())
        .args(["-c", "chk"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "no properly formatted SHA256 checksum lines found",
        ));
}
