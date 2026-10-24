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

// --- P1: escape sequences in SET1/SET2 (Round 12 F1) ------------------------

/// The classic `tr '\n' ' '` must actually see a newline in SET1, so
/// `printf 'a\nb\n' | tr '\n' ' '` prints `a b ` (trailing space, no final
/// newline). Pre-fix escapes were literal characters.
#[test]
fn escape_newline_translate() {
    cargo_bin_cmd!("tr")
        .args(["\\n", " "])
        .write_stdin(b"a\nb\n")
        .assert()
        .success()
        .stdout("a b ");
}

/// `\NNN` octal escapes resolve (`\101` = `A`).
#[test]
fn escape_octal() {
    cargo_bin_cmd!("tr")
        .args(["\\101", "X"])
        .write_stdin("A\n")
        .assert()
        .success()
        .stdout("X\n");
}

/// `\\` escapes a literal backslash.
#[test]
fn escape_backslash() {
    cargo_bin_cmd!("tr")
        .args(["\\\\", "/"])
        .write_stdin("a\\b\n")
        .assert()
        .success()
        .stdout("a/b\n");
}

// --- P1: -c in translate mode complements SET1 ------------------------------

/// `tr -c 'a' 'x'`: every character NOT in SET1 (including the newline)
/// becomes the last character of SET2; SET1 characters pass through.
/// Pre-fix the complement had no effect on translation.
#[test]
fn complement_translate_maps_non_set_chars_to_last_set2() {
    cargo_bin_cmd!("tr")
        .args(["-c", "a", "x"])
        .write_stdin("abc\n")
        .assert()
        .success()
        .stdout("axxx");
}

// --- P1: -s with two sets squeezes by SET2 after translation ----------------

/// `tr -s 'ab' 'xy'` translates then squeezes repeats of SET2 characters;
/// pre-fix it squeezed by SET1 (`cc` would have been squeezed too).
#[test]
fn squeeze_uses_set2_after_translation() {
    cargo_bin_cmd!("tr")
        .args(["-s", "ab", "xy"])
        .write_stdin("aabbccd\n")
        .assert()
        .success()
        .stdout("xyccd\n");
}

// --- P1: reversed ranges error with exit 1 ----------------------------------

/// `z-a` is a reversed range: GNU errors instead of silently producing an
/// empty set.
#[test]
fn reverse_range_errors_with_exit_1() {
    cargo_bin_cmd!("tr")
        .args(["z-a", "ab"])
        .write_stdin("z\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "reverse collating sequence order",
        ))
        .stderr(predicates::str::contains("panicked").not());
}

// --- P1: POSIX character classes --------------------------------------------

/// `tr '[:lower:]' '[:upper:]'` maps positionally across both classes.
#[test]
fn character_classes_translate_positionally() {
    cargo_bin_cmd!("tr")
        .args(["[:lower:]", "[:upper:]"])
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("HELLO WORLD\n");
}

/// An unknown class name is an error.
#[test]
fn unknown_character_class_errors() {
    cargo_bin_cmd!("tr")
        .args(["[:bogus:]", "x"])
        .write_stdin("a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("invalid character class"));
}
