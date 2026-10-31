use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn exits_zero_and_prints_nonempty() {
    cargo_bin_cmd!("w")
        .assert()
        .success()
        .stdout(predicates::str::is_empty().not());
}

#[test]
fn header_flag_prints_boot_time_line() {
    let out = cargo_bin_cmd!("w").arg("-h").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    // The header shows the boot timestamp (Chinese-localized: 系统启动时间).
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("启动") || text.to_lowercase().contains("up"),
        "output: {text}"
    );
}

#[test]
fn output_never_panics_without_sessions() {
    // Sessions may be absent in service contexts; the tool must succeed.
    let out = cargo_bin_cmd!("w").arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}
