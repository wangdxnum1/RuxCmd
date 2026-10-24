use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn no_operands_errors_instead_of_panicking() {
    // Regression: missing SET1 used to hit `expect("set1 is required")`.
    cargo_bin_cmd!("tr")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("tr: missing operand"))
        .stderr(predicates::str::contains(
            "Usage: tr [OPTION]... SET1 [SET2]",
        ))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn empty_set2_in_translate_mode_errors_instead_of_panicking() {
    // Regression: `tr 'a' ''` used to underflow `set2[set2_len - 1]`.
    cargo_bin_cmd!("tr")
        .args(["a", ""])
        .write_stdin("a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("tr:"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn empty_set2_with_squeeze_errors_too() {
    cargo_bin_cmd!("tr")
        .args(["-s", "a", ""])
        .write_stdin("aa\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("tr:"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn translate_mode_requires_set2() {
    cargo_bin_cmd!("tr")
        .args(["a"])
        .write_stdin("abc\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("tr: missing operand after 'a'"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn delete_mode_allows_missing_set2() {
    cargo_bin_cmd!("tr")
        .args(["-d", "a"])
        .write_stdin("banana\n")
        .assert()
        .success()
        .stdout("bnn\n");
}

#[test]
fn translate_replaces_characters() {
    cargo_bin_cmd!("tr")
        .args(["ab", "xy"])
        .write_stdin("abcabc\n")
        .assert()
        .success()
        .stdout("xycxyc\n");
}
