use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

// Regression for P0 #43: `-s` was declared on both --set and --seconds, so clap
// panicked ("Short option names must be unique") on every invocation, including
// --help, in debug builds. Every test below would have hit that panic before
// the fix because any argument parsing aborted with exit 101.

#[test]
fn help_runs_without_panicking_in_debug_builds() {
    cargo_bin_cmd!("date")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn plain_utc_display_runs_without_panicking_in_debug_builds() {
    cargo_bin_cmd!("date")
        .args(["-u"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not())
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn d_epoch_zero_prints_unix_epoch_utc() {
    cargo_bin_cmd!("date")
        .args(["-d", "@0", "-u", "+%Y-%m-%d"])
        .assert()
        .success()
        .stdout("1970-01-01\n");
}

#[test]
fn d_epoch_zero_default_format_contains_epoch_year() {
    cargo_bin_cmd!("date")
        .args(["-d", "@0", "-u"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1970"));
}

#[test]
fn d_iso_date_prints_requested_date_utc() {
    cargo_bin_cmd!("date")
        .args(["-d", "2026-10-07", "-u", "+%Y-%m-%d"])
        .assert()
        .success()
        .stdout("2026-10-07\n");
}

#[test]
fn d_iso_datetime_with_seconds_utc() {
    cargo_bin_cmd!("date")
        .args(["-d", "2026-10-07T12:34:56", "-u", "+%Y-%m-%dT%H:%M:%S"])
        .assert()
        .success()
        .stdout("2026-10-07T12:34:56\n");
}

#[test]
fn d_iso_datetime_with_minutes_only_utc() {
    cargo_bin_cmd!("date")
        .args(["-d", "2026-10-07T12:34", "-u", "+%Y-%m-%dT%H:%M"])
        .assert()
        .success()
        .stdout("2026-10-07T12:34\n");
}

#[test]
fn d_epoch_seconds_format_as_utc_time() {
    cargo_bin_cmd!("date")
        .args(["-d", "@1700000000", "-u", "+%Y-%m-%dT%H:%M:%S"])
        .assert()
        .success()
        .stdout("2023-11-14T22:13:20\n");
}

#[test]
fn d_garbage_fails_with_invalid_date_error() {
    cargo_bin_cmd!("date")
        .args(["-d", "garbage"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("date: invalid date 'garbage'"))
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn d_empty_string_means_now() {
    cargo_bin_cmd!("date")
        .args(["-d", ""])
        .assert()
        .success()
        .stderr(predicates::str::contains("panicked").not());
}

#[test]
fn positional_date_string_is_synonym_of_d() {
    let via_flag = cargo_bin_cmd!("date")
        .args(["-d", "2026-10-07", "-u"])
        .output()
        .unwrap();
    let via_positional = cargo_bin_cmd!("date")
        .args(["2026-10-07", "-u"])
        .output()
        .unwrap();
    assert!(via_flag.status.success(), "via -d: {:?}", via_flag);
    assert!(
        via_positional.status.success(),
        "via positional: {:?}",
        via_positional
    );
    assert_eq!(via_positional.stdout, via_flag.stdout);
    assert!(!via_positional.stdout.is_empty());
    // Default format shows the month/day/year of the requested date.
    let text = String::from_utf8(via_positional.stdout).unwrap();
    assert!(text.contains("Oct 07"), "stdout was: {}", text);
    assert!(text.contains("2026"), "stdout was: {}", text);
}

#[test]
fn seconds_long_name_still_works_with_epoch_operand() {
    // `-s` now belongs to --set only; the long --seconds flag keeps working and
    // applies to the resolved instant (like GNU `date -d @N +%s`).
    cargo_bin_cmd!("date")
        .args(["--seconds", "@1700000000", "-u"])
        .assert()
        .success()
        .stdout("1700000000\n");
}

#[test]
fn s_short_option_requires_a_value_like_set() {
    // `-s` maps to --set (requires a date value); the old bool --seconds took
    // none, so a bare -s failing with "a value is required" proves the mapping.
    cargo_bin_cmd!("date")
        .arg("-s")
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("value is required"));
}

#[test]
fn s_with_invalid_value_errors_without_touching_the_clock() {
    // Safe test for the --set path: the value is parsed before any SetLocalTime
    // call, so an invalid value never reaches the system-time API.
    cargo_bin_cmd!("date")
        .args(["-s", "garbage"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("date: invalid date 'garbage'"))
        .stderr(predicates::str::contains("panicked").not());
}
