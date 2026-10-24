//! Behavior tests for unexpand: GNU requires at least two blanks before a
//! run may become a tab (Round 12 F1).

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_unexpand"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn unexpand");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child
        .wait_with_output()
        .expect("failed to wait for unexpand")
}

// --- P1: a single space must stay a space, even on a tab stop --------------

/// `printf 'abc d' | unexpand -t 4` must emit `abc d` (the specified
/// acceptance case).
#[test]
fn single_space_stays_a_space() {
    let out = run(&["-t", "4"], b"abc d\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"abc d\n");
}

/// The actual regression: with `-a`, a lone space that lands exactly on a
/// tab stop must stay a space. Pre-fix `space_run >= 1` was always true and
/// `unexpand -a -t 4` emitted `abc\td`.
#[test]
fn single_space_on_tab_stop_with_all_flag_stays() {
    let out = run(&["-a", "-t", "4"], b"abc d\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"abc d\n");
}

/// A run of two spaces that reaches the tab stop still becomes a tab.
#[test]
fn two_space_run_at_tab_stop_becomes_tab() {
    let out = run(&["-a", "-t", "4"], b"ab  \n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"ab\t\n");
}

/// Regular leading-indent tabification keeps working.
#[test]
fn leading_indent_tabification_still_works() {
    let out = run(&["-t", "4"], b"    abc\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\tabc\n");
}
