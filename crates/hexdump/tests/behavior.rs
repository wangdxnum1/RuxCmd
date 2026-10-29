use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn run(args: &[&str], stdin_bytes: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("hexdump")
        .args(args)
        .write_stdin(stdin_bytes)
        .output()
        .unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn default_two_byte_hex_units() {
    let (out, _err, code) = run(&[], b"AB");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  4142 \n");
}

#[test]
fn default_partial_unit_padding() {
    let (out, _err, code) = run(&[], b"ABC");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  4142 43   \n");
}

#[test]
fn canonical_format_matches_bsd_layout() {
    let (out, _err, code) = run(&["-C"], b"ABC");
    assert_eq!(code, Some(0));
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(text.starts_with("00000000  41 42 43"), "got: {text:?}");
    assert!(text.contains("|ABC|"));
    assert!(text.ends_with("00000003\n"));
}

#[test]
fn char_mode_lists_characters() {
    let (out, _err, code) = run(&["-c"], b"ABC");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  A B C \n");
}

#[test]
fn decimal_mode_uses_five_wide_units() {
    // "AB" little... grouped as 0x4142 = 16706; the trailing single byte
    // 0x43 = 67 prints as a three-digit partial slot.
    let (out, _err, code) = run(&["-d"], b"ABC");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  16706 067   \n");
}

#[test]
fn octal_mode_uses_six_wide_units() {
    let (out, _err, code) = run(&["-o"], b"ABC");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  040502 103   \n");
}

#[test]
fn hex_mode_matches_default_grouping() {
    let (out, _err, code) = run(&["-x"], b"ABC");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  4142 43   \n");
}

#[test]
fn length_limits_consumed_bytes() {
    let (out, _err, code) = run(&["-n", "4"], b"ABCDEF");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000000  4142 4344 \n");
}

#[test]
fn skip_offsets_the_start() {
    let (out, _err, code) = run(&["-s", "4"], b"ABCDEF");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00000004  4546 \n");
}

#[test]
fn empty_input_prints_nothing() {
    let (out, _err, code) = run(&[], b"");
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
}

#[test]
fn file_operand_is_read() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.bin");
    fs::write(&f, b"AB").unwrap();
    let out = cargo_bin_cmd!("hexdump")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"00000000  4142 \n");
}

#[test]
fn missing_file_reports_error_and_exits_one() {
    let (_out, err, code) = run(&["missing.hexdump.bin"], b"");
    assert_eq!(code, Some(1));
    assert!(err.starts_with("hexdump: missing.hexdump.bin:"), "stderr: {}", err);
}
