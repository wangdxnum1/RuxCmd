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
