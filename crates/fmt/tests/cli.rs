//! Behavior tests for fmt: `-c` must be a valueless flag with crown-margin
//! semantics, the width must not underflow when an indent exceeds `-w`, and
//! `--split-only` must break at whitespace instead of mid-word
//! (Round 12 F1).

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fmt"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn fmt");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for fmt")
}

// --- P1: `fmt -c` takes no value (GNU crown margin) ------------------------

/// Pre-fix `-c/--crown-margin` was `Option<usize>` and `fmt -c` died in clap
/// with "a value is required".
#[test]
fn crown_margin_flag_takes_no_value() {
    let out = run(&["-c", "-w", "8"], b"aaaa bb cc dd\n");
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    assert_eq!(out.stdout, b"aaaa bb\ncc dd\n");
}

/// Crown margin preserves the input paragraph's leading indent.
#[test]
fn crown_margin_preserves_leading_indent() {
    let out = run(&["-c", "-w", "12"], b"    aa bb cc dd\n");
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    // Every output line keeps the 4-space indent; 8 columns remain for
    // words, so "aa bb cc" fills the first line.
    assert_eq!(out.stdout, b"    aa bb cc\n    dd\n");
}

// --- P1: width below indent must not underflow -----------------------------

/// `fmt -c -w 1` with a 2-space indent: pre-fix `width - indent` underflowed
/// usize and wrapping was silently disabled (or panicked in debug).
#[test]
fn width_below_indent_does_not_panic() {
    let out = run(&["-c", "-w", "1"], b"  abc def\n");
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("panicked"),
        "stderr: {:?}",
        out.stderr
    );
    assert_eq!(out.stdout, b"  abc\n  def\n");
}

// --- P1: --split-only must break at whitespace -----------------------------

/// Pre-fix `--split-only` cut at character positions, splitting words in
/// half (`ab cdef` / `g` / `h`).
#[test]
fn split_only_breaks_at_whitespace() {
    let out = run(&["-s", "-w", "8"], b"ab cdefgh\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"ab\ncdefgh\n");
}

/// A window without any whitespace still hard-cuts at a char boundary.
#[test]
fn split_only_hard_cuts_when_no_whitespace() {
    let out = run(&["-s", "-w", "4"], b"abcdefgh\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"abcd\nefgh\n");
}
