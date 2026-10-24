use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn print_type_shows_real_filesystem() {
    let out = cargo_bin_cmd!("df").args(["-T", "C:\\"]).assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("Type"), "header: {:?}", stdout);
    assert!(stdout.contains("NTFS"), "type column: {:?}", stdout);
}

#[test]
fn missing_path_reports_and_exits_one() {
    let out = cargo_bin_cmd!("df")
        .arg("Q:\\")
        .assert()
        .failure()
        .stderr(|s: &str| s.contains("cannot get filesystem statistics"));
    let _ = out;
}
