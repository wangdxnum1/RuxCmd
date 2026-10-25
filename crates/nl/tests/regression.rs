//! Regression tests for P0 #20 (Task 35): `nl` rebuilt every line with
//! `line_content.push(byte as char)` (Latin-1 mojibake for any non-ASCII
//! input), mapped the `-b` styles to the wrong meanings (`n` meant nonblank,
//! there was no `t`, the default was `a` instead of `t`), and dropped every
//! `\r` byte instead of just a trailing one.
//!
//! Expected outputs below were byte-verified against GNU coreutils nl 8.32:
//! unnumbered lines print `width + separator_len` spaces (the separator is
//! blanked out too), a blank line does not consume a line number under `t`,
//! and a missing final newline is not invented.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str;

fn nl_args(args: &[&str], input: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("nl").args(args).write_stdin(input).assert()
}

/// Unnumbered line under the default `-w 6 -s \t`: 7 spaces, no tab.
const BLANK: &str = "       \n";

// --- P0: UTF-8 must pass through verbatim -----------------------------------

/// Task 35 spec case: UTF-8 (Chinese) lines are numbered without mojibake.
#[test]
fn utf8_lines_are_numbered_verbatim() {
    nl_args(&[], "你好\nworld\n")
        .success()
        .stdout("     1\t你好\n     2\tworld\n");
}

/// Multi-byte content with an explicit style must survive too.
#[test]
fn utf8_lines_with_style_a_verbatim() {
    nl_args(&["-b", "a"], "café\n\nnaïve\n")
        .success()
        .stdout("     1\tcafé\n     2\t\n     3\tnaïve\n");
}

// --- P0/P1: -b styles and the GNU/POSIX default ------------------------------

/// Task 35 spec case: default (style `t`) numbers non-empty lines only; the
/// blank line is not numbered and does not consume a number (GNU: `1 a`,
/// blank, `2 b`).
#[test]
fn default_numbers_nonempty_lines_only() {
    nl_args(&[], "a\n\nb\n").success().stdout(format!(
        "     1\ta\n{}\n     2\tb\n",
        BLANK.trim_end_matches('\n')
    ));
}

/// A line holding a single space counts as non-empty under `t` (GNU behavior).
#[test]
fn space_only_line_is_numbered() {
    nl_args(&[], " \n").success().stdout("     1\t \n");
}

/// Task 35 spec case: `-b a` numbers blank lines too.
#[test]
fn style_a_numbers_blank_lines() {
    nl_args(&["-b", "a"], "a\n\nb\n")
        .success()
        .stdout("     1\ta\n     2\t\n     3\tb\n");
}

/// `-b n` numbers nothing: every line gets the blanked number field.
#[test]
fn style_n_numbers_nothing() {
    nl_args(&["-b", "n"], "a\n\nb\n")
        .success()
        .stdout("       a\n       \n       b\n");
}

/// `-b t` is accepted and identical to the default.
#[test]
fn style_t_matches_default() {
    nl_args(&["-b", "t"], "a\n\nb\n").success().stdout(format!(
        "     1\ta\n{}\n     2\tb\n",
        BLANK.trim_end_matches('\n')
    ));
}

/// Unknown style is rejected with a GNU-style error message, exit code 1.
#[test]
fn invalid_style_rejected() {
    cargo_bin_cmd!("nl")
        .args(["-b", "x"])
        .write_stdin("a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(str::contains("invalid body numbering style"));
}

// --- P0-adjacent: \r handling -------------------------------------------------

/// Only a *trailing* \r (CRLF input) is dropped; interior \r is content.
#[test]
fn interior_cr_is_preserved() {
    nl_args(&[], "a\rb\n").success().stdout("     1\ta\rb\n");
}

/// CRLF input lines are treated as normal lines.
#[test]
fn crlf_lines_have_trailing_cr_stripped() {
    nl_args(&[], "a\r\nb\r\n")
        .success()
        .stdout("     1\ta\n     2\tb\n");
}

/// A CRLF blank line is an unnumbered blank under the default style.
#[test]
fn crlf_blank_line_is_unnumbered() {
    nl_args(&[], "a\r\n\r\nb\r\n").success().stdout(format!(
        "     1\ta\n{}\n     2\tb\n",
        BLANK.trim_end_matches('\n')
    ));
}

// --- guards -------------------------------------------------------------------

/// Input without a final newline: GNU nl terminates the output line anyway
/// (Round 13 difftest finding).
#[test]
fn missing_final_newline_is_terminated_like_gnu() {
    nl_args(&[], "a\nb")
        .success()
        .stdout("     1\ta\n     2\tb\n");
}

/// Empty input produces empty output.
#[test]
fn empty_input_empty_output() {
    nl_args(&[], "").success().stdout("");
}
