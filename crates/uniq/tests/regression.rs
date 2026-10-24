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
