use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const SHA1_HELLO: &str = "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d";
const SHA1_EMPTY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

#[test]
fn check_accepts_binary_star_marker() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.bin"), b"hello").unwrap();
    fs::write(dir.path().join("chk"), format!("{SHA1_HELLO} *data.bin\n")).unwrap();
    cargo_bin_cmd!("sha1sum")
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
        format!("\u{feff}{SHA1_HELLO} f.txt\n"),
    )
    .unwrap();
    cargo_bin_cmd!("sha1sum")
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
        format!("{} f1.txt\n", "0".repeat(40)),
    )
    .unwrap();
    fs::write(dir.path().join("c2.txt"), format!("{SHA1_HELLO} f1.txt\n")).unwrap();
    cargo_bin_cmd!("sha1sum")
        .current_dir(dir.path())
        .args(["-c", "c1.txt", "c2.txt"])
        .assert()
        .code(1)
        .stdout("f1.txt: OK\n")
        .stderr("f1.txt: FAILED\nsha1sum: WARNING: 1 computed checksum did NOT match\n");
}

#[test]
fn tag_format_for_file_and_stdin() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    cargo_bin_cmd!("sha1sum")
        .current_dir(dir.path())
        .args(["--tag", "f.txt"])
        .assert()
        .success()
        .stdout(format!("SHA1 (f.txt) = {SHA1_HELLO}\n"));
    cargo_bin_cmd!("sha1sum")
        .args(["--tag"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(format!("SHA1 (-) = {SHA1_EMPTY}\n"));
}

#[test]
fn supports_dash_stdin() {
    cargo_bin_cmd!("sha1sum")
        .arg("-")
        .write_stdin("hello")
        .assert()
        .success()
        .stdout(format!("{SHA1_HELLO}  -\n"));
}

#[test]
fn status_is_silent_and_reports_exit_code_only() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("f.txt"), b"hello").unwrap();
    fs::write(
        dir.path().join("bad"),
        format!("{} f.txt\n", "0".repeat(40)),
    )
    .unwrap();
    fs::write(dir.path().join("good"), format!("{SHA1_HELLO} f.txt\n")).unwrap();
    cargo_bin_cmd!("sha1sum")
        .current_dir(dir.path())
        .args(["--status", "-c", "bad"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
    cargo_bin_cmd!("sha1sum")
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
    cargo_bin_cmd!("sha1sum")
        .current_dir(dir.path())
        .args(["-c", "chk"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "no properly formatted SHA1 checksum lines found",
        ));
}
