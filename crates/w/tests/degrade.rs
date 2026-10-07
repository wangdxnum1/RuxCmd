use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn broken_path_degrades_without_panicking() {
    // Regression: w used to panic via .expect() when one of its external
    // sources (systeminfo / query user / whoami) could not be launched.
    // With a broken PATH it must keep running, degrade the missing fields
    // to "unavailable", and exit 0.
    cargo_bin_cmd!("w")
        .env("PATH", "/nonexistent")
        .assert()
        .success()
        .stderr(predicates::str::contains("panicked").not());
}
