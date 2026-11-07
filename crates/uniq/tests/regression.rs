//! Regression tests for P0 #18 (Task 34): `uniq` stored the comparison key
//! produced by `make_key` in `prev_line` and printed that key, so `uniq -i`
//! lower-cased the entire output and `uniq -f 1` truncated output lines to
//! their post-skip remainder. The fix keeps the original line for output and
//! compares via the key only.

use assert_cmd::cargo::cargo_bin_cmd;

fn uniq_args(args: &[&str], input: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("uniq")
        .args(args)
        .write_stdin(input)
        .assert()
}

// --- output must be the original line, not the comparison key --------------

/// Task 34 spec case: `uniq -i` collapses case-identical adjacent lines but
/// must print the first original line with its case intact (GNU: `Hello`).
#[test]
fn ignore_case_preserves_original_case() {
    uniq_args(&["-i"], "Hello\nhello\n")
        .success()
        .stdout("Hello\n");
}

/// With `-i` the first-seen spelling is the one printed verbatim.
#[test]
fn ignore_case_prints_first_spelling_verbatim() {
    uniq_args(&["-i"], "hello\nHELLO\n")
        .success()
        .stdout("hello\n");
}

/// Task 34 spec case: `uniq -f 1` compares lines without the first field but
/// must print the complete original line (`x aa`, not the truncated `aa`).
#[test]
fn skip_fields_prints_full_original_line() {
    uniq_args(&["-f", "1"], "x aa\ny aa\n")
        .success()
        .stdout("x aa\n");
}

/// Combined `-f 1 -c`: full original line after the count prefix.
#[test]
fn skip_fields_with_count_prints_full_line() {
    uniq_args(&["-f", "1", "-c"], "x aa\ny aa\n")
        .success()
        .stdout("      2 x aa\n");
}

// --- unchanged behavior guards ----------------------------------------------

/// Plain uniq and the `-c` `%7ld ` prefix format (main.rs:94) stay as before.
#[test]
fn count_prefix_format_unchanged() {
    uniq_args(&["-c"], "a\na\nb\n")
        .success()
        .stdout("      2 a\n      1 b\n");
}

/// `-d`/`-u` selection is untouched by the storage fix.
#[test]
fn duplicate_and_unique_filters_unchanged() {
    uniq_args(&["-d"], "a\na\nb\n").success().stdout("a\n");
    uniq_args(&["-u"], "a\na\nb\n").success().stdout("b\n");
}

/// Plain uniq over mixed input still collapses only exact repeats.
#[test]
fn plain_uniq_collapses_exact_repeats() {
    uniq_args(&[], "a\na\nb\na\n").success().stdout("a\nb\na\n");
}

// --- second operand: write filtered output to a file (GNU uniq IN OUT) ------

/// `uniq IN OUT` writes the result to OUT instead of stdout, created or
/// truncated like GNU.
#[test]
fn output_operand_writes_filtered_lines() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.txt");
    let output = dir.path().join("out.txt");
    std::fs::write(&input, "a\na\nb\nc\n").unwrap();
    cargo_bin_cmd!("uniq")
        .arg(&input)
        .arg(&output)
        .assert()
        .success()
        .stdout("");
    assert_eq!(std::fs::read(&output).unwrap(), b"a\nb\nc\n");
}

/// Options combine with the output operand (`uniq -c IN OUT`).
#[test]
fn output_operand_with_count() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.txt");
    let output = dir.path().join("out.txt");
    std::fs::write(&input, "a\na\nb\n").unwrap();
    cargo_bin_cmd!("uniq")
        .arg("-c")
        .arg(&input)
        .arg(&output)
        .assert()
        .success();
    assert_eq!(std::fs::read(&output).unwrap(), b"      2 a\n      1 b\n");
}

/// An unwritable output path is reported and exits 1.
#[test]
fn unwritable_output_operand_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.txt");
    let missing_dir = dir.path().join("no-such-dir").join("out.txt");
    std::fs::write(&input, "a\n").unwrap();
    cargo_bin_cmd!("uniq")
        .arg(&input)
        .arg(&missing_dir)
        .assert()
        .failure()
        .stderr(predicates::str::contains("uniq:"));
}

/// A trailing `-` operand still reads stdin and writes to stdout.
#[test]
fn dash_operand_unchanged() {
    cargo_bin_cmd!("uniq")
        .arg("-")
        .write_stdin("x\nx\ny\n")
        .assert()
        .success()
        .stdout("x\ny\n");
}
