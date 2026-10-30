use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn exits_zero_with_or_without_sessions() {
    // Service sessions may have no interactive sessions; the tool must
    // succeed either way and never panic.
    cargo_bin_cmd!("who").assert().success();
}

#[test]
fn boot_flag_prints_a_boot_line() {
    let out = cargo_bin_cmd!("who").arg("-b").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.to_lowercase().contains("boot"),
        "expected a boot line, got: {text}"
    );
}

#[test]
fn count_flag_prints_a_number() {
    let out = cargo_bin_cmd!("who").arg("--count").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(text.parse::<u32>().is_ok(), "count output: {text:?}");
}
