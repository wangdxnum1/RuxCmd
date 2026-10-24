//! P1 regression tests (Round 12 D1): cut must be byte-faithful —
//! non-UTF-8 data passes through, -b emits raw bytes, and trailing
//! spaces survive (only \r is stripped from CRLF lines).

use std::path::PathBuf;
use std::process::Command;

fn cut(args: &[&str], file: &std::path::Path) -> (Vec<u8>, Option<i32>) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cut"));
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(file);
    let output = cmd.output().expect("cut runs");
    (output.stdout, output.status.code())
}

fn write_file(name: &str, data: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ruxcmd-cut-test-{}-{}", std::process::id(), name));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(name);
    std::fs::write(&path, data).unwrap();
    path
}

// --- P1: -b selects raw bytes (no as-char re-encoding) ---

#[test]
fn multibyte_bytes_selection_outputs_raw_bytes() {
    // "你好世界\n" — 12 bytes of UTF-8 plus newline; -b 1-3 must yield the
    // first three raw bytes (exactly the encoding of 你).
    let path = write_file("utf8.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-3"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你\n".as_bytes());
}

#[test]
fn non_utf8_bytes_pass_through() {
    // A line that is not valid UTF-8 must survive selection untouched.
    let path = write_file("bin.txt", b"abc\xFFdef\nplain\n");
    let (stdout, code) = cut(&["-b", "2-4"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"bc\xFF\nlai\n");
}

// --- P1: non-UTF-8 field selection is not silently truncated ---

#[test]
fn non_utf8_field_selection_still_sees_later_lines() {
    // Before the fix, read_line errors swallowed the rest of the input;
    // the second line must still be processed.
    let path = write_file("bin2.txt", b"caf\xFF,x\nplain,z\n");
    let (stdout, code) = cut(&["-d", ",", "-f", "2"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"x\nz\n");
}

// --- P1: trailing spaces are preserved (only \r is stripped) ---

#[test]
fn trailing_spaces_are_preserved() {
    let path = write_file("spaces.txt", b"hello   \nworld \n");
    let (stdout, code) = cut(&[], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"hello   \nworld \n");
}

#[test]
fn crlf_is_stripped_but_trailing_tabs_kept() {
    let path = write_file("crlf.txt", b"a,b\r\nc,d\r\n");
    let (stdout, code) = cut(&["-d", ",", "-f", "1"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"a\nc\n");
}
