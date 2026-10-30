use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn small_composite_factors() {
    cargo_bin_cmd!("factor")
        .arg("12")
        .assert()
        .success()
        .stdout("12: 2 2 3\n");
}

#[test]
fn prime_prints_itself() {
    cargo_bin_cmd!("factor")
        .arg("97")
        .assert()
        .success()
        .stdout("97: 97\n");
}

#[test]
fn one_prints_bare_label() {
    cargo_bin_cmd!("factor")
        .arg("1")
        .assert()
        .success()
        .stdout("1:\n");
}

#[test]
fn zero_prints_bare_label_like_gnu() {
    cargo_bin_cmd!("factor")
        .arg("0")
        .assert()
        .success()
        .stdout("0:\n");
}

#[test]
fn u64_max_factors_completely() {
    cargo_bin_cmd!("factor")
        .arg("18446744073709551615")
        .assert()
        .success()
        .stdout("18446744073709551615: 3 5 17 257 641 65537 6700417\n");
}

#[test]
fn invalid_number_reports_and_exits_one() {
    cargo_bin_cmd!("factor")
        .arg("12x")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("factor: '12x' is not a valid positive integer"));
}

#[test]
fn stdin_numbers_are_factored_line_by_line() {
    cargo_bin_cmd!("factor")
        .write_stdin("12\n0\n7\n")
        .assert()
        .success()
        .stdout("12: 2 2 3\n0:\n7: 7\n");
}

#[test]
fn multiple_operands_each_get_a_line() {
    cargo_bin_cmd!("factor")
        .args(["4", "9"])
        .assert()
        .success()
        .stdout("4: 2 2\n9: 3 3\n");
}
