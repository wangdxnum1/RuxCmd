use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn matching_lines_and_exit_codes() {
    cargo_bin_cmd!("grep")
        .arg("needle")
        .write_stdin("hay\nneedle\n")
        .assert()
        .success()
        .stdout("needle\n");
    cargo_bin_cmd!("grep")
        .arg("missing")
        .write_stdin("hay\nneedle\n")
        .assert()
        .code(1)
        .stdout("");
}

#[test]
fn count_on_single_file_prints_number() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "apple\nbanana\napple pie\ncherry\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-c")
        .arg("apple")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("2\n");

    cargo_bin_cmd!("grep")
        .arg("-c")
        .arg("pear")
        .arg(&path)
        .assert()
        .code(1)
        .stdout("0\n");
}
