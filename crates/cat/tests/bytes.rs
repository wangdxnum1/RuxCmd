//! Byte-level regression tests: cat must be a byte-transparent pipe by default
//! and must not corrupt CRLF, non-ASCII UTF-8, or binary data (P0 #1).

use assert_cmd::cargo::cargo_bin_cmd;
use std::path::PathBuf;

fn write_file(dir: &tempfile::TempDir, name: &str, data: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, data).unwrap();
    path
}

fn cat_bytes(args: &[&str], file: Option<&PathBuf>) -> Vec<u8> {
    let mut cmd = cargo_bin_cmd!("cat");
    for arg in args {
        cmd.arg(arg);
    }
    if let Some(file) = file {
        cmd.arg(file);
    }
    cmd.assert().success();
    cmd.assert().get_output().stdout.clone()
}

// --- Default (no options): byte-exact passthrough ---

#[test]
fn utf8_file_passes_through_byte_exact() {
    let dir = tempfile::tempdir().unwrap();
    let data = "你好，世界\nこんにちは\ncafé résumé\n".as_bytes();
    let file = write_file(&dir, "utf8.txt", data);
    assert_eq!(cat_bytes(&[], Some(&file)), data);
}

#[test]
fn utf8_stdin_passes_through_byte_exact() {
    let data = "中文输入\n通过管道\n".as_bytes();
    let mut cmd = cargo_bin_cmd!("cat");
    cmd.write_stdin(data);
    cmd.assert().success();
    assert_eq!(cmd.assert().get_output().stdout.clone(), data);
}

#[test]
fn binary_data_passes_through_file_byte_exact() {
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(b"text\r\nlines\r\n");
    data.extend_from_slice(&[0x00, 0x01, 0x07, 0x08]);
    data.extend_from_slice(&[0x0B, 0x0C, 0x1B]);
    data.extend_from_slice(&[0x80, 0x9F, 0xC3, 0xA9, 0xFE, 0xFF]);
    data.extend_from_slice(b"\r\n\r");
    data.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "binary.dat", &data);
    assert_eq!(cat_bytes(&[], Some(&file)), data);
}

#[test]
fn binary_data_passes_through_stdin_byte_exact() {
    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(b"a\r\nb\r");
    data.extend_from_slice(&[0x00, 0x80, 0xFF]);
    let mut cmd = cargo_bin_cmd!("cat");
    cmd.write_stdin(data.clone());
    cmd.assert().success();
    assert_eq!(cmd.assert().get_output().stdout.clone(), data);
}

#[test]
fn crlf_line_endings_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "crlf.txt", b"alpha\r\nbeta\r\n");
    assert_eq!(cat_bytes(&[], Some(&file)), b"alpha\r\nbeta\r\n");
}

#[test]
fn multiple_files_concatenate_byte_exact() {
    let dir = tempfile::tempdir().unwrap();
    let a = write_file(&dir, "a.txt", b"abc"); // no trailing newline
    let b = write_file(&dir, "b.txt", "你好\n".as_bytes());
    assert_eq!(
        cat_bytes(&[a.to_str().unwrap(), b.to_str().unwrap()], None),
        b"abc\xe4\xbd\xa0\xe5\xa5\xbd\n"
    );
}

// --- -E / -A / -T / -v: GNU display semantics at the byte level ---

#[test]
fn show_ends_marks_line_ends_and_keeps_cr_raw() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "crlf.txt", b"a\r\nb\n");
    assert_eq!(cat_bytes(&["-E"], Some(&file)), b"a\r$\nb$\n");
}

#[test]
fn show_ends_high_bytes_stay_raw() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "utf8.txt", "é\n".as_bytes()); // C3 A9
    assert_eq!(cat_bytes(&["-E"], Some(&file)), b"\xc3\xa9$\n");
}

#[test]
fn show_all_displays_cr_as_caret_m_and_tab_as_caret_i() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "crlf.txt", b"a\r\nb\tc\n");
    assert_eq!(cat_bytes(&["-A"], Some(&file)), b"a^M$\nb^Ic$\n");
}

#[test]
fn show_nonprinting_displays_controls_and_high_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "ctrl.bin", b"\x01\x7f\r\t\n\x80\xff");
    // GNU cat -v: control chars -> ^X, DEL -> ^?, high bytes -> M- notation;
    // TAB and LF pass through unchanged.
    assert_eq!(cat_bytes(&["-v"], Some(&file)), b"^A^?^M\t\nM-^@M-^?");
}

#[test]
fn show_tabs_displays_tab_and_keeps_other_bytes_raw() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "tabs.txt", b"x\ty\r\nz\n");
    assert_eq!(cat_bytes(&["-T"], Some(&file)), b"x^Iy\r\nz\n");
}

#[test]
fn show_ends_does_not_mark_unterminated_last_line() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "noeol.txt", b"no-newline");
    assert_eq!(cat_bytes(&["-E"], Some(&file)), b"no-newline");
}

// --- -n / -b / -s semantics (guards against rewrite regressions) ---

#[test]
fn number_counts_all_lines_including_empty() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "lines.txt", b"a\n\nb\n");
    assert_eq!(
        cat_bytes(&["-n"], Some(&file)),
        b"     1\ta\n     2\t\n     3\tb\n"
    );
}

#[test]
fn number_works_on_crlf_and_keeps_cr() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "crlf.txt", b"a\r\n\r\nb\r\n");
    assert_eq!(
        cat_bytes(&["-n"], Some(&file)),
        b"     1\ta\r\n     2\t\r\n     3\tb\r\n"
    );
}

#[test]
fn number_counts_unterminated_last_line() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "noeol.txt", b"abc");
    assert_eq!(cat_bytes(&["-n"], Some(&file)), b"     1\tabc");
}

#[test]
fn number_nonblank_skips_empty_lines() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "lines.txt", b"a\n\nb\n");
    assert_eq!(cat_bytes(&["-b"], Some(&file)), b"     1\ta\n\n     2\tb\n");
}

#[test]
fn squeeze_blank_collapses_runs() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "blank.txt", b"a\n\n\n\nb\n");
    assert_eq!(cat_bytes(&["-s"], Some(&file)), b"a\n\nb\n");
}

#[test]
fn squeeze_blank_with_show_ends() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_file(&dir, "blank.txt", b"a\n\n\n\nb\n");
    assert_eq!(cat_bytes(&["-s", "-E"], Some(&file)), b"a$\n$\nb$\n");
}
