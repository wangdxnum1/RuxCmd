use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn emits_clear_screen_and_home_sequence() {
    let out = cargo_bin_cmd!("clear").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\x1b[2J\x1b[H");
}

#[test]
fn version_flag_exits_zero() {
    cargo_bin_cmd!("clear")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("clear"));
}

