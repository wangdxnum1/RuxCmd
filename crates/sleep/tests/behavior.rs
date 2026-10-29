use assert_cmd::cargo::cargo_bin_cmd;
use std::time::Instant;

#[test]
fn sleeps_for_the_sum_of_operands() {
    // 0.1s + 0.1s must sleep about 0.2s: less than 0.1s means the operands
    // were not summed, more than 2s means something is badly wrong.
    let start = Instant::now();
    cargo_bin_cmd!("sleep")
        .args(["0.1", "0.1"])
        .assert()
        .success();
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 180, "elapsed: {:?}", elapsed);
    assert!(elapsed.as_secs() < 2, "elapsed: {:?}", elapsed);
}

#[test]
fn invalid_interval_reports_and_exits_one() {
    cargo_bin_cmd!("sleep")
        .arg("xx")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "sleep: invalid time interval 'xx'",
        ));
}

#[test]
fn uppercase_suffix_is_rejected_like_gnu() {
    cargo_bin_cmd!("sleep")
        .arg("1S")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("invalid time interval"));
}

#[test]
fn negative_operand_is_rejected() {
    cargo_bin_cmd!("sleep")
        .arg("-1")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn zero_sleeps_not_at_all() {
    let start = Instant::now();
    cargo_bin_cmd!("sleep").arg("0").assert().success();
    assert!(start.elapsed().as_millis() < 500);
}

#[test]
fn missing_operand_is_a_usage_error() {
    cargo_bin_cmd!("sleep")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("missing operand"));
}

#[test]
fn version_and_help_flags_work() {
    cargo_bin_cmd!("sleep")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("sleep"));

    cargo_bin_cmd!("sleep")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"));
}
