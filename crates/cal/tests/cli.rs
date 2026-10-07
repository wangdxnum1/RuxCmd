use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn single_argument_above_twelve_is_treated_as_year() {
    // Regression: `cal 2024` used to be parsed as month=2024, from_ymd_opt
    // returned None and the unwrap chain panicked.
    cargo_bin_cmd!("cal")
        .args(["2024"])
        .assert()
        .success()
        .stdout(predicates::str::contains("January 2024"))
        .stdout(predicates::str::contains("December 2024"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn single_argument_thirteen_is_rejected() {
    // Regression: `cal 13` used to panic (month=13 -> from_ymd_opt None).
    cargo_bin_cmd!("cal")
        .args(["13"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cal: invalid month '13'"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn single_argument_zero_is_rejected() {
    cargo_bin_cmd!("cal")
        .args(["0"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cal: invalid month '0'"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn month_and_year_prints_single_month() {
    cargo_bin_cmd!("cal")
        .args(["7", "2024"])
        .assert()
        .success()
        .stdout(predicates::str::contains("July 2024"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn month_out_of_range_with_year_is_rejected() {
    cargo_bin_cmd!("cal")
        .args(["13", "2024"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cal: invalid month '13'"));
}

#[test]
fn year_out_of_range_is_rejected() {
    cargo_bin_cmd!("cal")
        .args(["10000"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cal: invalid year '10000'"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn current_month_still_works() {
    cargo_bin_cmd!("cal").assert().success();
}
