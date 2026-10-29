use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn run(args: &[&str], stdin_bytes: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("strings")
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
fn default_minimum_length_is_four() {
    let (out, _err, code) = run(&[], b"ab\ncdef\nxy\nabcdefgh\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"cdef\nabcdefgh\n");
}

#[test]
fn custom_minimum_length() {
    let (out, _err, code) = run(&["-n", "2"], b"ab\ncdef\nxy\nabcdefgh\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"ab\ncdef\nxy\nabcdefgh\n");
}

#[test]
fn tab_counts_as_printable_like_binutils() {
    // binutils strings treats TAB as part of a printable run.
    let (out, _err, code) = run(&[], b"a\tb\tcd\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a\tb\tcd\n");
}

#[test]
fn non_printable_bytes_terminate_runs() {
    let (out, _err, code) = run(&[], b"abcd\x00\x01efgh\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"abcd\nefgh\n");
}

#[test]
fn decimal_offsets_are_seven_wide_space_padded() {
    let (out, _err, code) = run(&["-t", "d"], b"abcdef\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"      0 abcdef\n");
}

#[test]
fn hex_offsets_are_seven_wide_space_padded() {
    let (out, _err, code) = run(&["-t", "x"], b"\x00\x00\x00abcdef\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"      3 abcdef\n");
}

#[test]
fn file_name_prefix_uses_operand_with_space() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("f.txt");
    fs::write(&f, b"abcdef\n").unwrap();
    let out = cargo_bin_cmd!("strings")
        .arg("-f")
        .arg(f.to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let expected = format!("{}: abcdef\n", f.to_string_lossy());
    assert_eq!(out.stdout, expected.as_bytes());
}

#[test]
fn eight_bit_encoding_emits_raw_high_bytes() {
    // An invalid sequence must not become U+FFFD replacement characters.
    let (out, _err, code) = run(&["-e", "S"], &[0x41, 0xFF, 0xFE, 0x42, b'\n']);
    assert_eq!(code, Some(0));
    assert_eq!(out, vec![0x41, 0xFF, 0xFE, 0x42, b'\n']);
}

#[test]
fn seven_bit_default_skips_high_bytes() {
    let (out, _err, code) = run(&[], &[0x41, 0xC3, 0xA9, 0x42, b'\n']);
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
}

#[test]
fn missing_file_reports_error_and_exits_one() {
    let (_out, err, code) = run(&["definitely-missing.bin"], b"");
    assert_eq!(code, Some(1));
    assert!(err.contains("strings:"), "stderr: {}", err);
}

#[test]
fn multiple_files_are_processed_in_order() {
    let dir = TempDir::new().unwrap();
    let f1 = dir.path().join("one.bin");
    let f2 = dir.path().join("two.bin");
    fs::write(&f1, b"first string\n").unwrap();
    fs::write(&f2, b"second string\n").unwrap();
    let out = cargo_bin_cmd!("strings")
        .args([
            "-f",
            f1.to_string_lossy().as_ref(),
            f2.to_string_lossy().as_ref(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let expected = format!(
        "{}: first string\n{}: second string\n",
        f1.to_string_lossy(),
        f2.to_string_lossy()
    );
    assert_eq!(out.stdout, expected.as_bytes());
}
