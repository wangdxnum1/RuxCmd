use std::process::Command;

fn run(args: &[&str]) -> (Vec<u8>, Vec<u8>, Option<i32>) {
    let out = Command::new(env!("CARGO_BIN_EXE_printf"))
        .args(args)
        .output()
        .expect("failed to run printf");
    (out.stdout, out.stderr, out.status.code())
}

#[test]
fn interprets_newline_escape_in_format() {
    // GNU printf adds no trailing newline of its own: `printf "a\nb"`
    // writes exactly `a`, LF, `b`.
    let (out, err, code) = run(&["a\\nb"]);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(out, b"a\nb");
}

#[test]
fn interprets_tab_hex_and_octal_escapes() {
    // \t, \xHH (up to two hex digits) and \0NNN (up to three octal digits
    // after the zero), matching the echo -e escape family.
    let (out, _, code) = run(&["t\\tx\\x41\\0101"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"t\txAA");
}

#[test]
fn keeps_unknown_escapes_verbatim() {
    let (out, _, code) = run(&["a\\qb"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a\\qb");
}

#[test]
fn escapes_backslash_and_percent_literal() {
    let (out, _, code) = run(&["\\\\100%%"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"\\100%");
}

#[test]
fn missing_arguments_become_empty_or_zero() {
    let (out, err, code) = run(&["%s-%d", "x"]);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(out, b"x-0");

    let (out, _, code) = run(&["[%s][%d][%x][%f]"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"[][0][0][0]");
}

#[test]
fn invalid_numeric_argument_reports_error_and_exits_1() {
    // GNU coreutils substitutes 0 for the unconvertible value, reports the
    // error on stderr and exits 1 after processing the whole format.
    let (out, err, code) = run(&["%d", "abc"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, b"0");
    let msg = String::from_utf8_lossy(&err);
    assert!(msg.contains("expected a numeric value"), "stderr: {msg}");
}

#[test]
fn invalid_numeric_argument_continues_with_remaining_args() {
    let (out, err, code) = run(&["%d-%d", "abc", "5"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, b"0-5");
    let msg = String::from_utf8_lossy(&err);
    assert!(msg.contains("expected a numeric value"), "stderr: {msg}");
}

#[test]
fn invalid_hex_argument_exits_1() {
    let (out, _, code) = run(&["%x", "zz"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, b"0");
}
