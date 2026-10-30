use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_known_variable_value() {
    cargo_bin_cmd!("printenv")
        .env("RUXCMD_TEST_VAR", "hello-value")
        .arg("RUXCMD_TEST_VAR")
        .assert()
        .success()
        .stdout("hello-value\n");
}

#[test]
fn missing_variable_exits_one() {
    cargo_bin_cmd!("printenv")
        .arg("RUXCMD_DEFINITELY_MISSING_VAR_XYZ")
        .assert()
        .failure()
        .code(1)
        .stdout("");
}

#[test]
fn no_names_prints_whole_environment() {
    cargo_bin_cmd!("printenv")
        .env("RUXCMD_TEST_VAR", "yes")
        .assert()
        .success()
        .stdout(predicates::str::contains("RUXCMD_TEST_VAR=yes"));
}

#[test]
fn null_separator() {
    cargo_bin_cmd!("printenv")
        .env("RUXCMD_TEST_VAR", "yes")
        .args(["-0", "RUXCMD_TEST_VAR"])
        .assert()
        .success()
        .stdout("yes\0");
}

#[test]
fn any_missing_name_fails_even_with_hits() {
    cargo_bin_cmd!("printenv")
        .env("RUXCMD_TEST_VAR", "yes")
        .args(["RUXCMD_TEST_VAR", "RUXCMD_MISSING_XYZ"])
        .assert()
        .failure()
        .code(1)
        .stdout("yes\n");
}
