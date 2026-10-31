use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn substitutes_dollar_var() {
    cargo_bin_cmd!("envsubst")
        .env("MY_TEST_VAR", "hello")
        .write_stdin("$MY_TEST_VAR\n")
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn substitutes_brace_var() {
    cargo_bin_cmd!("envsubst")
        .env("MY_TEST_VAR", "world")
        .write_stdin("${MY_TEST_VAR}!\n")
        .assert()
        .success()
        .stdout("world!\n");
}

#[test]
fn unknown_var_becomes_empty() {
    cargo_bin_cmd!("envsubst")
        .write_stdin("$DEFINITELY_MISSING_XYZ\n")
        .assert()
        .success()
        .stdout("\n");
}

#[test]
fn plain_text_passes_through() {
    cargo_bin_cmd!("envsubst")
        .write_stdin("no variables here\n")
        .assert()
        .success()
        .stdout("no variables here\n");
}

#[test]
fn shell_format_restricts_substitution() {
    cargo_bin_cmd!("envsubst")
        .env("A1", "yes")
        .env("B2", "no")
        .arg("$A1")
        .write_stdin("$A1 $B2\n")
        .assert()
        .success()
        .stdout("yes $B2\n");
}
