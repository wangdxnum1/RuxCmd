use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_a_positive_count() {
    let out = cargo_bin_cmd!("nproc").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.trim().parse::<u32>().unwrap() >= 1);
}

#[test]
fn ignore_zero_matches_default() {
    let a = cargo_bin_cmd!("nproc").output().unwrap();
    let b = cargo_bin_cmd!("nproc")
        .args(["--ignore=0"])
        .output()
        .unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn all_flag_prints_total_processors() {
    cargo_bin_cmd!("nproc")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

#[test]
fn huge_ignore_clamps_to_one() {
    let out = cargo_bin_cmd!("nproc")
        .args(["--ignore=100000"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}
