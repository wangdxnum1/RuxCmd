//! Round 12 batch E2: P1 remediation tests for awk.
//! Covers: if/else evaluated against the original condition tokens, print
//! with comma-separated arguments (OFS) and numeric/string/literal
//! arguments, `next` skipping the remaining rules for the current record,
//! a single BEGIN/END run with NR continuing across files, no $N leakage
//! between records, awk pattern truthiness, and `for` rejected loudly at
//! parse time. Std-only on purpose (no new dependencies).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_awk"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // awk may exit early (parse error) before reading stdin; ignore EPIPE.
    let _ = child.stdin.as_mut().unwrap().write_all(stdin);
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("awk-e2-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_bytes(path: &Path, data: &[u8]) {
    std::fs::write(path, data).unwrap();
}

// --- P1: if/else must evaluate the original condition tokens ---

#[test]
fn if_else_compares_fields_numerically() {
    let out = awk(
        &["{if ($1 > $2) print \"gt\"; else print \"le\"}"],
        b"5 3\n",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"gt\n");

    let out = awk(
        &["{if ($1 > $2) print \"gt\"; else print \"le\"}"],
        b"3 5\n",
    );
    assert_eq!(out.stdout, b"le\n");
}

#[test]
fn if_condition_with_number_and_equality() {
    let out = awk(&["{if ($1 == 2) print \"two\"}"], b"1\n2\n");
    assert_eq!(out.stdout, b"two\n");
}

// --- P1: print with multiple comma-separated arguments ---

#[test]
fn print_joins_arguments_with_ofs() {
    // On the second record $2 is empty; OFS still separates the arguments.
    let out = awk(&["{print $1, $2}"], b"a b\nc\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a b\nc \n");
}

#[test]
fn print_outputs_numbers_strings_and_paren_form() {
    let out = awk(&["{print 42}"], b"x\n");
    assert_eq!(out.stdout, b"42\n");

    let out = awk(&["{print($1)}"], b"hi\n");
    assert_eq!(out.stdout, b"hi\n");

    let out = awk(&["{print \"lit\"}"], b"x\n");
    assert_eq!(out.stdout, b"lit\n");
}

// --- P1: next skips the remaining rules for the current record ---

#[test]
fn next_skips_remaining_statements_in_rule() {
    let out = awk(&["{if ($1 == 2) next; print}"], b"1\n2\n3\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"1\n3\n");
}

#[test]
fn next_in_one_rule_skips_later_rules() {
    let out = awk(&["{next} {print}"], b"a\nb\n");
    assert_eq!(out.stdout, b"");
}

// --- P1: one BEGIN/END run, NR continues across files ---

#[test]
fn begin_end_run_once_and_nr_continues_across_files() {
    let dir = temp_dir("nrcross");
    let f1 = dir.join("f1.txt");
    let f2 = dir.join("f2.txt");
    write_bytes(&f1, b"a\nb\n");
    write_bytes(&f2, b"c\nd\n");

    let out = awk(
        &[
            "BEGIN{print \"B\"} {print NR} END{print \"E\"}",
            f1.to_str().unwrap(),
            f2.to_str().unwrap(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"B\n1\n2\n3\n4\nE\n");
}

// --- P1: stale $N fields must not leak into shorter records ---

#[test]
fn stale_fields_cleared_for_shorter_records() {
    let out = awk(&["{print $3}"], b"a b c\nX\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"c\n\n");
}

#[test]
fn nf_reflects_current_record() {
    let out = awk(&["{print NF}"], b"a b c\nX\n");
    assert_eq!(out.stdout, b"3\n1\n");
}

// --- P1: pattern truthiness follows awk value rules ---

#[test]
fn pattern_truthiness_follows_awk_value_rules() {
    // non-zero numeric field -> true; "0" and empty -> false
    let out = awk(&["$1 {print NR}"], b"5\n0\n\n");
    assert_eq!(out.stdout, b"1\n");

    // non-empty string other than "0" -> true
    let out = awk(&["BEGIN{x=\"abc\"} x{print \"t\"}"], b"a\n");
    assert_eq!(out.stdout, b"t\n");

    // "0" as a value -> false
    let out = awk(&["BEGIN{x=\"0\"} x{print \"t\"}"], b"a\n");
    assert_eq!(out.stdout, b"");
}

#[test]
fn non_numeric_string_is_zero_in_comparisons() {
    let out = awk(
        &["{if ($1 > 0) print \"pos\"; else print \"nonpos\"}"],
        b"abc\n",
    );
    assert_eq!(out.stdout, b"nonpos\n");
}

#[test]
fn expression_patterns_parse_as_single_rule() {
    // `$1 == "a" { ... }` must be one rule, not `$1 -> print` plus the action.
    let out = awk(&["$1 == \"a\" {print \"A\"}"], b"a\nb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"A\n");

    // pattern without an action defaults to printing matching records
    let out = awk(&["$1 > 1"], b"1\n2\n3\n");
    assert_eq!(out.stdout, b"2\n3\n");
}

// --- P1: for is rejected loudly at parse time (exit 1, English stderr) ---

#[test]
fn for_loop_executes_three_part_form() {
    // Round 21: classic for loops run with postfix increments.
    let out = awk(&["BEGIN { for (i = 0; i < 3; i++) print i }"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n1\n2\n");
}

// --- printf keeps working after the argument-collection refactor ---

#[test]
fn printf_formats_and_percent_escape() {
    let out = awk(&["{printf \"%s-%d\\n\", $1, $2}"], b"a 7\n");
    assert_eq!(out.stdout, b"a-7\n");

    let out = awk(&["BEGIN{printf \"100%%\\n\"}"], b"");
    assert_eq!(out.stdout, b"100%\n");
}
