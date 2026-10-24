//! Regression tests for P0 #17: `-k`/`-t` were parsed by clap but never used
//! by the implementation (silent no-op), `-u` deduplicated *before* sorting
//! (only removing adjacent duplicates), and `-c -r` checked the same
//! ascending condition in both branches so reverse-ordered input was
//! reported as disorder.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str;
use std::fs;

fn sort_args(args: &[&str], input: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("sort")
        .args(args)
        .write_stdin(input)
        .assert()
}

// --- -t / -k field selection ------------------------------------------------

/// Task 33 spec case: `sort -t, -k2 -n` over `b,2 a,1 c,3` must sort by the
/// numeric value of field 2.
#[test]
fn t_comma_k2_n_sorts_by_second_field() {
    sort_args(&["-t,", "-k2", "-n"], "b,2\na,1\nc,3\n")
        .success()
        .stdout("a,1\nb,2\nc,3\n");
}

/// Distinguishing case: field-2 numeric order (1, 2, 10) differs from
/// whole-line order. Before the fix `-k`/`-t` were ignored, so the input was
/// sorted by whole line and printed `a,10 b,2 c,1`.
#[test]
fn k2_n_numeric_ignores_non_key_prefix() {
    sort_args(&["-t,", "-k", "2", "-n"], "b,2\na,10\nc,1\n")
        .success()
        .stdout("c,1\nb,2\na,10\n");
}

/// `sort -k2` without -n must compare lexicographically on field 2. The
/// expected order (field 2: alpha, beta, gamma) differs from whole-line order.
#[test]
fn k2_lexicographic_sorts_by_second_field() {
    sort_args(&["-k2"], "zeta beta\nalpha gamma\nmid alpha\n")
        .success()
        .stdout("mid alpha\nzeta beta\nalpha gamma\n");
}

/// Without -t the key is still dictionary-ordered: "10" < "2" byte-wise.
#[test]
fn t_comma_k2_lexicographic_is_not_numeric() {
    sort_args(&["-t,", "-k2"], "a,10\na,2\n")
        .success()
        .stdout("a,10\na,2\n");
}

/// `-k n,m` limits the key to fields n..=m: with `-k2,2` the two lines below
/// have equal keys, so `-u` collapses them to one (the first in input order,
/// since the stable sort performs no last-resort comparison under -u); with
/// `-k2` (through end of line) the keys "9 a" and "9 b" differ and both
/// lines survive.
#[test]
fn k_with_end_field_limits_the_key() {
    sort_args(&["-u", "-k", "2,2"], "a 9 b\na 9 a\n")
        .success()
        .stdout("a 9 b\n");
    sort_args(&["-u", "-k2"], "a 9 b\na 9 a\n")
        .success()
        .stdout("a 9 a\na 9 b\n");
}

/// With -t a leading separator creates an empty first field (GNU behavior);
/// an empty field has no leading number, so it sorts as 0.
#[test]
fn t_leading_separator_makes_empty_first_field() {
    sort_args(&["-t,", "-k1", "-n"], ",5\n1\n")
        .success()
        .stdout(",5\n1\n");
}

/// Fields beyond the end of the line produce an empty key: all three lines
/// tie on field 3 and the GNU last-resort whole-line comparison keeps
/// dictionary order.
#[test]
fn k_field_beyond_line_end_yields_empty_key() {
    sort_args(&["-k", "3"], "b x\na y\nc z\n")
        .success()
        .stdout("a y\nb x\nc z\n");
}

/// -r works together with -k/-t/-n.
#[test]
fn reverse_with_key_reverses_field_order() {
    sort_args(&["-r", "-t,", "-k2", "-n"], "b,2\na,1\nc,3\n")
        .success()
        .stdout("c,3\nb,2\na,1\n");
}

/// -k/-t still work when input comes from a file argument.
#[test]
fn key_sort_works_with_file_argument() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rows.csv");
    fs::write(&path, "b,2\na,10\nc,1\n").unwrap();
    cargo_bin_cmd!("sort")
        .args(["-t,", "-k2", "-n"])
        .arg(&path)
        .assert()
        .success()
        .stdout("c,1\nb,2\na,10\n");
}

// --- -u ----------------------------------------------------------------------

/// P0 #17 (2): `-u` must deduplicate *after* sorting. `b a b` has no adjacent
/// duplicates, so the pre-fix order (dedup, then sort) printed two `b`s.
#[test]
fn u_dedups_after_sorting() {
    sort_args(&["-u"], "b\na\nb\n").success().stdout("a\nb\n");
}

/// Adjacent duplicates are still removed.
#[test]
fn u_removes_adjacent_duplicates() {
    sort_args(&["-u"], "a\na\nb\n").success().stdout("a\nb\n");
}

// --- -c ----------------------------------------------------------------------

/// P0 #17 (3): `-c -r` must accept reverse-ordered input. The pre-fix code
/// checked the ascending condition in both branches.
#[test]
fn check_reverse_accepts_reverse_ordered_input() {
    sort_args(&["-c", "-r"], "c\nb\na\n").success();
}

#[test]
fn check_reverse_rejects_input_that_is_not_reverse_ordered() {
    sort_args(&["-c", "-r"], "b\na\nc\n")
        .failure()
        .code(1)
        .stderr(str::contains("disorder"));
}

#[test]
fn check_accepts_sorted_input() {
    sort_args(&["-c"], "a\nb\nc\n").success();
}

#[test]
fn check_rejects_unsorted_input() {
    sort_args(&["-c"], "a\nc\nb\n")
        .failure()
        .code(1)
        .stderr(str::contains("disorder"));
}

// --- option validation --------------------------------------------------------

/// A field separator must be a single character (Task 33: multi-character
/// `-t` is rejected with exit 1 instead of being silently accepted or
/// rejected as a generic clap type error).
#[test]
fn t_rejects_multi_character_separator() {
    sort_args(&["-t", "ab"], "a\n")
        .failure()
        .code(1)
        .stderr(str::contains("multi-character field separator"));
}

#[test]
fn k_rejects_invalid_specification() {
    sort_args(&["-k", "0"], "a\n")
        .failure()
        .code(1)
        .stderr(str::contains("invalid field specification"));
    sort_args(&["-k", "x"], "a\n")
        .failure()
        .code(1)
        .stderr(str::contains("invalid field specification"));
    sort_args(&["-k", "2,1"], "a\n")
        .failure()
        .code(1)
        .stderr(str::contains("invalid field specification"));
}

// --- -n / -h numeric parsing ---------------------------------------------------

/// -n compares by the leading number of the key (optional sign and decimal
/// fraction), not by requiring the whole line to be a valid f64.
#[test]
fn n_uses_leading_number_prefix() {
    sort_args(&["-n"], "10 apples\n9 bananas\n")
        .success()
        .stdout("9 bananas\n10 apples\n");
}

#[test]
fn n_handles_signs_and_decimals() {
    sort_args(&["-n"], "-3\n+2\n5\n")
        .success()
        .stdout("-3\n+2\n5\n");
    sort_args(&["-n"], "1.5\n1.10\n")
        .success()
        .stdout("1.10\n1.5\n");
}

/// P2 follow-up: -h must use 1024 as base, so 1023 < 1K(=1024) < 1M.
/// Pre-fix (1000 base) 1K sorted as 1000 and came before 1023.
#[test]
fn h_uses_1024_base() {
    sort_args(&["-h"], "1M\n1023\n1K\n")
        .success()
        .stdout("1023\n1K\n1M\n");
}

/// P2 follow-up: -h supports the full K M G T P E Z Y suffix set.
/// 1P = 1024^5 < 1E = 1024^6, which whole-line string order gets backwards.
#[test]
fn h_supports_peta_zetta_yotta_suffixes() {
    sort_args(&["-h"], "1E\n1P\n").success().stdout("1P\n1E\n");
    sort_args(&["-h"], "1Y\n1Z\n").success().stdout("1Z\n1Y\n");
}

// --- P1: byte transparency, exit codes, `-` stdin (Round 12 D2) -------------

/// Runs sort with a file operand and returns raw stdout plus the exit code,
/// so non-UTF-8 output can be compared byte for byte.
fn sort_stdout_bytes(file: &std::path::Path) -> (Vec<u8>, Option<i32>) {
    let output = cargo_bin_cmd!("sort")
        .arg(file)
        .assert()
        .get_output()
        .clone();
    (output.stdout.clone(), output.status.code())
}

/// Lines that are not valid UTF-8 must be sorted (on their lossy text) and
/// echoed with their original bytes; the pre-fix `read_line` replaced
/// invalid bytes with U+FFFD on output.
#[test]
fn non_utf8_lines_keep_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bin.txt");
    let data = b"\xFF banana\napple\n\xFE cherry\n";
    fs::write(&path, data).unwrap();
    let (stdout, code) = sort_stdout_bytes(&path);
    assert_eq!(code, Some(0));
    // The lossy text of both marker lines starts with U+FFFD, so both sort
    // after "apple" and order between themselves by the readable remainder.
    assert_eq!(stdout, b"apple\n\xFF banana\n\xFE cherry\n");
}

/// Trailing blanks are part of the line: only the `\r` of a CRLF terminator
/// is stripped, other trailing whitespace survives (pre-fix `trim_end()`
/// stripped spaces and tabs from every line).
#[test]
fn trailing_whitespace_is_preserved() {
    sort_args(&[], "b \na\n").success().stdout("a\nb \n");
    sort_args(&[], "b\t\na\n").success().stdout("a\nb\t\n");
}

/// CRLF terminators are still removed from the sort key and the output.
#[test]
fn crlf_terminators_are_stripped() {
    sort_args(&[], "b\r\na\r\n").success().stdout("a\nb\n");
}

/// A file that cannot be opened is reported and makes sort exit 2 (GNU),
/// while the readable operands are still processed and printed.
#[test]
fn unopenable_file_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");
    let ok = dir.path().join("ok.txt");
    fs::write(&ok, "b\na\n").unwrap();
    cargo_bin_cmd!("sort")
        .arg(&missing)
        .arg(&ok)
        .assert()
        .code(2)
        .stdout("a\nb\n")
        .stderr(str::contains("missing.txt"));
}

/// `sort -` reads standard input (GNU operand semantics).
#[test]
fn dash_operand_reads_stdin() {
    sort_args(&["-"], "b\na\nc\n").success().stdout("a\nb\nc\n");
}
