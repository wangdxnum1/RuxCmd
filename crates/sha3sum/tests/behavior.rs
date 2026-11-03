use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn sha3_256_of_hello() {
    let out = cargo_bin_cmd!("sha3sum")
        .args(["--bits", "256"])
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    let digest = text.split("  ").next().unwrap();
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn sha3_512_default() {
    let out = cargo_bin_cmd!("sha3sum")
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    let digest = text.split("  ").next().unwrap();
    assert_eq!(digest.len(), 128);
}

#[test]
fn bits_224() {
    let out = cargo_bin_cmd!("sha3sum")
        .args(["--bits", "224"])
        .write_stdin(b"x")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let digest = text.split("  ").next().unwrap();
    assert_eq!(digest.len(), 56);
}

#[test]
fn file_operand_and_two_space_format() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.txt");
    fs::write(&f, b"test data\n").unwrap();
    let out = cargo_bin_cmd!("sha3sum")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("  "), "output: {text}");
    assert!(text.ends_with("data.txt\n"), "output: {text}");
}

#[test]
fn missing_file_fails() {
    cargo_bin_cmd!("sha3sum")
        .arg("definitely-missing.bin")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn invalid_bits_fails() {
    cargo_bin_cmd!("sha3sum")
        .args(["--bits", "100"])
        .assert()
        .failure();
}
