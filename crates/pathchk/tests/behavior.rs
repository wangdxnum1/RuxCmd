use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn valid_path_is_silent_success() {
    cargo_bin_cmd!("pathchk")
        .arg("C:/some/normal/path.txt")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn oversized_component_fails() {
    let long = "x".repeat(300);
    cargo_bin_cmd!("pathchk")
        .arg(format!("C:/dir/{}", long))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("exceeding 255"));
}

#[test]
fn empty_operand_fails() {
    cargo_bin_cmd!("pathchk").arg("").assert().failure().code(1);
}

#[test]
fn portability_rejects_special_characters() {
    cargo_bin_cmd!("pathchk")
        .args(["-p", "C:/dir/a b"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("portable filename set"));

    cargo_bin_cmd!("pathchk")
        .args(["-p", "C:/dir/ok_name-2.txt"])
        .assert()
        .success();
}

#[test]
fn all_flag_rejects_leading_dash() {
    cargo_bin_cmd!("pathchk")
        .args(["-P", "-dash"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn missing_operand_is_an_error() {
    cargo_bin_cmd!("pathchk")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing operand"));
}
