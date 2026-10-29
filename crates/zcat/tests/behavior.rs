use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

// gzip -c fixture bytes generated with Python's gzip module.
const HELLO_GZ: &[u8] = &[
    31, 139, 8, 0, 106, 212, 227, 106, 2, 255, 203, 72, 205, 201, 201, 87, 168, 74, 78, 44, 225, 2,
    0, 163, 163, 125, 137, 11, 0, 0, 0,
];
const TWO_LINES_GZ: &[u8] = &[
    31, 139, 8, 0, 106, 212, 227, 106, 2, 255, 75, 203, 44, 42, 46, 225, 42, 78, 77, 206, 207, 75,
    225, 2, 0, 32, 81, 69, 8, 13, 0, 0, 0,
];
const EMPTY_GZ: &[u8] = &[
    31, 139, 8, 0, 106, 212, 227, 106, 2, 255, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn run_stdin(bytes: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("zcat").write_stdin(bytes).output().unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn decompresses_stdin_stream() {
    let (out, err, code) = run_stdin(HELLO_GZ);
    assert_eq!(code, Some(0));
    assert!(err.is_empty());
    assert_eq!(out, b"hello zcat\n");
}

#[test]
fn decompresses_file_operand() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.txt.gz");
    fs::write(&f, TWO_LINES_GZ).unwrap();
    let out = cargo_bin_cmd!("zcat")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"first\nsecond\n");
}

#[test]
fn empty_stream_decompresses_to_nothing() {
    let (out, _err, code) = run_stdin(EMPTY_GZ);
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
}

#[test]
fn non_gzip_input_fails_with_error() {
    let (_out, err, code) = run_stdin(b"plain text, not gzip\n");
    assert_eq!(code, Some(1));
    assert!(!err.is_empty());
}

#[test]
fn truncated_stream_fails() {
    let (_out, err, code) = run_stdin(&HELLO_GZ[..15]);
    assert_eq!(code, Some(1));
    assert!(!err.is_empty());
}

#[test]
fn missing_file_fails() {
    cargo_bin_cmd!("zcat")
        .arg("definitely-missing.txt.gz")
        .assert()
        .failure()
        .code(1);
}
