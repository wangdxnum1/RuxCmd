use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn maxdepth_zero_lists_only_start_point() {
    let out = cargo_bin_cmd!("find")
        .args(["--maxdepth", "0", "."])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "stdout: {:?}", stdout);
    assert!(lines[0].contains('.'), "stdout: {:?}", stdout);
}

#[test]
fn maxdepth_one_includes_direct_children() {
    let out = cargo_bin_cmd!("find")
        .args(["--maxdepth", "1", "."])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("src"), "stdout: {:?}", stdout);
}

#[test]
fn unknown_type_value_errors() {
    cargo_bin_cmd!("find")
        .args(["-type", "q", "."])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid argument"));
}

#[test]
fn malformed_glob_does_not_panic() {
    let out = cargo_bin_cmd!("find")
        .args(["-name", "a[", "."])
        .assert()
        .success();
    assert!(out.get_output().stdout.is_empty());
}

#[test]
fn gnu_single_dash_long_options_work() {
    let out = cargo_bin_cmd!("find")
        .args(["-maxdepth", "0", "-name", "Cargo.toml", "."])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert_eq!(stdout.lines().count(), 1, "stdout: {:?}", stdout);
}
