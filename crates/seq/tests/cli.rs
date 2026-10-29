use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn non_numeric_argument_errors_instead_of_panicking() {
    // Regression: `seq abc` used to panic with exit 101 (unwrap on parse).
    cargo_bin_cmd!("seq")
        .args(["abc"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seq: invalid floating point argument: 'abc'",
        ))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn second_argument_errors_instead_of_panicking() {
    cargo_bin_cmd!("seq")
        .args(["1", "x"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seq: invalid floating point argument: 'x'",
        ))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn third_argument_errors_instead_of_panicking() {
    cargo_bin_cmd!("seq")
        .args(["1", "2", "zz"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seq: invalid floating point argument: 'zz'",
        ))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn two_arguments_print_sequence() {
    cargo_bin_cmd!("seq")
        .args(["1", "3"])
        .assert()
        .success()
        .stdout("1\n2\n3\n");
}

#[test]
fn three_arguments_print_sequence() {
    cargo_bin_cmd!("seq")
        .args(["1", "2", "5"])
        .assert()
        .success()
        .stdout("1\n3\n5\n");
}

#[test]
fn negative_operands_after_flags_are_accepted() {
    // Regression: `seq -w -3 0` used to be rejected by clap treating -3 as
    // an unknown option; GNU accepts negative operands anywhere.
    cargo_bin_cmd!("seq")
        .args(["-w", "-3", "0"])
        .assert()
        .success()
        .stdout("-3\n-2\n-1\n00\n");
}

#[test]
fn negative_first_operand_without_flags() {
    cargo_bin_cmd!("seq")
        .args(["-3", "0"])
        .assert()
        .success()
        .stdout("-3\n-2\n-1\n0\n");
}
