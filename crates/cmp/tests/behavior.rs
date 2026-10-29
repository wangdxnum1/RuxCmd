use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, bytes: &[u8]) {
    fs::write(dir.path().join(name), bytes).unwrap();
}

fn run(dir: &TempDir, args: &[&str]) -> (String, String, Option<i32>) {
    let out = cargo_bin_cmd!("cmp")
        .args(args)
        .current_dir(dir.path())
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn identical_files_exit_zero_silently() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"hello\nworld\n");
    write(&dir, "b", b"hello\nworld\n");
    let (out, err, code) = run(&dir, &["a", "b"]);
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn differ_message_goes_to_stdout_with_char_and_line() {
    // GNU: "a b differ: char 4, line 2" on stdout, exit 1, empty stderr.
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"ab\ncd\n");
    write(&dir, "b", b"ab\nxd\n");
    let (out, err, code) = run(&dir, &["a", "b"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "a b differ: char 4, line 2\n");
    assert!(err.is_empty());
}

#[test]
fn differ_on_first_byte_reports_line_one() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"\x00ab\n");
    write(&dir, "b", b"\x01ab\n");
    let (out, _err, code) = run(&dir, &["a", "b"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "a b differ: char 1, line 1\n");
}

#[test]
fn eof_message_reports_byte_and_line() {
    // GNU: "cmp: EOF on 's1' after byte 3, in line 1" on stderr, exit 1.
    let dir = TempDir::new().unwrap();
    write(&dir, "s1", b"abc");
    write(&dir, "s2", b"abcx");
    let (_out, err, code) = run(&dir, &["s1", "s2"]);
    assert_eq!(code, Some(1));
    assert_eq!(err, "cmp: EOF on 's1' after byte 3, in line 1\n");
}

#[test]
fn eof_message_names_the_shorter_second_file() {
    let dir = TempDir::new().unwrap();
    write(&dir, "s1", b"abcx");
    write(&dir, "s2", b"abc");
    let (_out, err, code) = run(&dir, &["s1", "s2"]);
    assert_eq!(code, Some(1));
    assert_eq!(err, "cmp: EOF on 's2' after byte 3, in line 1\n");
}

#[test]
fn empty_file_uses_gnu_empty_wording() {
    let dir = TempDir::new().unwrap();
    write(&dir, "empty", b"");
    write(&dir, "a", b"x");
    let (out, err, code) = run(&dir, &["empty", "a"]);
    assert_eq!(code, Some(1));
    assert!(out.is_empty());
    assert_eq!(err, "cmp: EOF on 'empty' which is empty\n");
}

#[test]
fn silent_mode_prints_nothing() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"one\n");
    write(&dir, "b", b"two\n");
    let (out, err, code) = run(&dir, &["-s", "a", "b"]);
    assert_eq!(code, Some(1));
    assert!(out.is_empty());
    assert!(err.is_empty());
}

#[test]
fn verbose_lists_octal_byte_values() {
    // GNU -l: "4 143 170" (1-based position, octal byte values), exit 1.
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"ab\ncd\n");
    write(&dir, "b", b"ab\nxd\n");
    let (out, _err, code) = run(&dir, &["-l", "a", "b"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "4 143 170\n");
}

#[test]
fn list_and_silent_flags_conflict_with_exit_two() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"x");
    write(&dir, "b", b"y");
    let (_out, err, code) = run(&dir, &["-l", "-s", "a", "b"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("options -l and -s are incompatible"));
}

#[test]
fn missing_file_exits_two() {
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"x");
    let (_out, err, code) = run(&dir, &["nope", "a"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("cmp: nope"));
}

#[test]
fn eof_after_newline_drops_the_in_prefix() {
    // GNU: when the last compared byte is a newline the message reads
    // "after byte 3, line 1" instead of "in line 1".
    let dir = TempDir::new().unwrap();
    write(&dir, "n1", b"ab\n");
    write(&dir, "n2", b"ab\nc");
    let (_out, err, code) = run(&dir, &["n1", "n2"]);
    assert_eq!(code, Some(1));
    assert_eq!(err, "cmp: EOF on 'n1' after byte 3, line 1\n");
}

#[test]
fn single_operand_compares_standard_input() {
    // GNU: a missing second operand means '-'; empty stdin yields the
    // "which is empty" wording against '-'.
    let dir = TempDir::new().unwrap();
    write(&dir, "a", b"x");
    let out = cargo_bin_cmd!("cmp")
        .arg("a")
        .current_dir(dir.path())
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(err, "cmp: EOF on '-' which is empty\n");
}

#[test]
fn no_operands_is_a_usage_error_with_exit_two() {
    let out = cargo_bin_cmd!("cmp").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("missing operand"));
}

#[test]
fn binary_bytes_are_compared_raw() {
    let dir = TempDir::new().unwrap();
    write(&dir, "bin1", &[0x00, 0x01, 0xFF, b'\n']);
    write(&dir, "bin2", &[0x00, 0x01, 0xFE, b'\n']);
    let (out, _err, code) = run(&dir, &["bin1", "bin2"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "bin1 bin2 differ: char 3, line 1\n");
}
