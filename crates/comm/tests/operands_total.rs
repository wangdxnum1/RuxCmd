//! 0.4.6 GNU-alignment tests: `-` operands (standard input), `--total`, and
//! the `-1`/`-2`/`-3` TAB-prefix compaction compared byte-for-byte against
//! GNU coreutils 8.32 (`comm 8.32`, measured locally).
//!
//! GNU reference data (a.txt = A,B,C / b.txt = B,C,D):
//! - `comm a b`        -> `A\n\t\tB\n\t\tC\n\tD\n`
//! - `comm -1 a b`     -> `\tB\n\tC\nD\n`
//! - `comm -2 a b`     -> `A\n\tB\n\tC\n`
//! - `comm -3 a b`     -> `A\n\tD\n`
//! - `comm -12 a b`    -> `B\nC\n`
//! - `comm -13 a b`    -> `D\n`
//! - `comm -23 a b`    -> `A\n`
//! - `comm -123 a b`   -> empty
//! - `comm --total a b` -> data + `1\t1\t2\ttotal\n`; the counts are the
//!   unsuppressed per-column counts and the line is emitted even for empty
//!   inputs (`0\t0\t0\ttotal\n`) or with `-123` (`1\t1\t2\ttotal\n`).

use assert_cmd::cargo::cargo_bin_cmd;
use std::path::{Path, PathBuf};

fn files() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    std::fs::write(&a, b"A\nB\nC\n").unwrap();
    std::fs::write(&b, b"B\nC\nD\n").unwrap();
    (dir, a, b)
}

/// Run comm with the given args plus two file operands; return stdout.
fn run(args: &[&str], file1: &Path, file2: &Path) -> (Vec<u8>, Option<i32>) {
    let out = cargo_bin_cmd!("comm")
        .args(args)
        .arg(file1)
        .arg(file2)
        .output()
        .unwrap();
    (out.stdout, out.status.code())
}

/// Run comm with the given args and stdin data.
fn run_stdin(args: &[&str], stdin: &[u8]) -> (Vec<u8>, Option<i32>, String) {
    let out = cargo_bin_cmd!("comm")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap();
    (
        out.stdout,
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// --- `-1`/`-2`/`-3` TAB-prefix compaction, byte-for-byte vs GNU 8.32 -------

#[test]
fn tab_prefix_compaction_matches_gnu() {
    let (_dir, a, b) = files();
    let cases: &[(&[&str], &[u8])] = &[
        (&[], b"A\n\t\tB\n\t\tC\n\tD\n"),
        (&["-1"], b"\tB\n\tC\nD\n"),
        (&["-2"], b"A\n\tB\n\tC\n"),
        (&["-3"], b"A\n\tD\n"),
        (&["-12"], b"B\nC\n"),
        (&["-13"], b"D\n"),
        (&["-23"], b"A\n"),
        (&["-123"], b""),
    ];
    for (flags, expected) in cases {
        let (stdout, code) = run(flags, &a, &b);
        assert_eq!(code, Some(0), "flags {flags:?}");
        assert_eq!(stdout, *expected, "flags {flags:?}");
    }
}

// --- `-` operand: standard input on either side ---------------------------

/// `comm - FILE2` reads FILE1 from stdin (GNU: "When FILE1 or FILE2 (not
/// both) is -, read standard input").
#[test]
fn dash_as_first_operand_reads_stdin() {
    let (_dir, _a, b) = files();
    let (stdout, code, _stderr) = run_stdin(&["-", b.to_str().unwrap()], b"A\nB\nC\n");
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"A\n\t\tB\n\t\tC\n\tD\n");
}

/// `comm FILE1 -` reads FILE2 from stdin.
#[test]
fn dash_as_second_operand_reads_stdin() {
    let (_dir, a, _b) = files();
    let (stdout, code, _stderr) = run_stdin(&[a.to_str().unwrap(), "-"], b"B\nC\nD\n");
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"A\n\t\tB\n\t\tC\n\tD\n");
}

#[test]
fn dash_operand_respects_hide_flags() {
    let (_dir, _a, b) = files();
    let (stdout, code, _stderr) = run_stdin(&["-1", "-3", "-", b.to_str().unwrap()], b"A\nB\nC\n");
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"D\n");
}

/// Two `-` operands are rejected: a single stdin stream cannot be consumed
/// twice as two independent files.
#[test]
fn two_dash_operands_are_rejected() {
    let out = cargo_bin_cmd!("comm")
        .arg("-")
        .arg("-")
        .write_stdin(b"A\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("comm:"), "stderr: {stderr}");
}

// --- `--total` ------------------------------------------------------------

#[test]
fn total_appends_unsuppressed_counts() {
    let (_dir, a, b) = files();
    let (stdout, code) = run(&["--total"], &a, &b);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"A\n\t\tB\n\t\tC\n\tD\n1\t1\t2\ttotal\n");
}

/// The total counts are the full per-column counts even when columns are
/// suppressed (GNU increments counters before suppression).
#[test]
fn total_counts_ignore_suppression() {
    let (_dir, a, b) = files();
    for flags in [
        vec!["--total", "-1"],
        vec!["--total", "-2"],
        vec!["--total", "-3"],
        vec!["--total", "-12"],
        vec!["--total", "-123"],
    ] {
        let (stdout, code) = run_with_flags(&flags, &a, &b);
        assert_eq!(code, Some(0), "flags {flags:?}");
        assert!(
            stdout.ends_with(b"1\t1\t2\ttotal\n"),
            "flags {flags:?}, stdout {stdout:?}"
        );
    }
}

fn run_with_flags(flags: &[&str], file1: &Path, file2: &Path) -> (Vec<u8>, Option<i32>) {
    let out = cargo_bin_cmd!("comm")
        .args(flags)
        .arg(file1)
        .arg(file2)
        .output()
        .unwrap();
    (out.stdout, out.status.code())
}

/// Empty inputs still produce a `0\t0\t0\ttotal` line.
#[test]
fn total_on_empty_inputs_is_all_zero() {
    let dir = tempfile::tempdir().unwrap();
    let e = dir.path().join("empty");
    std::fs::write(&e, b"").unwrap();
    let (stdout, code) = run(&["--total"], &e, &e);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"0\t0\t0\ttotal\n");
}

/// In `-z` mode the total line keeps its literal `\t` column separators but
/// ends with the NUL record separator (GNU measured:
/// `1\t0\t0\ttotal\0`).
#[test]
fn total_in_zero_terminated_mode_uses_nul_terminator() {
    let dir = tempfile::tempdir().unwrap();
    let e = dir.path().join("empty");
    std::fs::write(&e, b"").unwrap();
    let (stdout, code, _stderr) =
        run_stdin(&["-z", "--total", "-", e.to_str().unwrap()], b"B\nC\nD\n");
    assert_eq!(code, Some(0));
    assert_eq!(
        stdout,
        [b"B\nC\nD\n\0".as_slice(), b"1\t0\t0\ttotal\0"].concat()
    );
}
