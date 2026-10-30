use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn encodes_known_vector() {
    cargo_bin_cmd!("base32")
        .write_stdin(b"hello\n")
        .assert()
        .success()
        .stdout("NBSWY3DPBI======\n");
}

#[test]
fn decodes_back_to_original() {
    let encoded = "NBSWY3DPBI======\n";
    cargo_bin_cmd!("base32")
        .args(["-d"])
        .write_stdin(encoded)
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn wrap_zero_emits_single_line() {
    cargo_bin_cmd!("base32")
        .args(["-w", "0"])
        .write_stdin(b"abc")
        .assert()
        .success()
        .stdout("MFRGG===\n");
}

#[test]
fn wrap_two_splits_lines() {
    let out = cargo_bin_cmd!("base32")
        .args(["-w", "2"])
        .write_stdin(b"abc")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text, "MF\nRG\nG=\n==\n");
}

#[test]
fn invalid_input_fails_without_ignore_garbage() {
    cargo_bin_cmd!("base32")
        .args(["-d"])
        .write_stdin(b"!!not-base32!!")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn ignore_garbage_skips_stray_characters() {
    cargo_bin_cmd!("base32")
        .args(["-d", "-i"])
        .write_stdin(b">>NBSWY3DPBI======<<")
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn file_operand_round_trips() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.bin");
    fs::write(&f, b"binary \x00 payload").unwrap();
    let out = cargo_bin_cmd!("base32")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

    let out = cargo_bin_cmd!("base32")
        .args(["-d"])
        .write_stdin(out.stdout)
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"binary \x00 payload");
}
