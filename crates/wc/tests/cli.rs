use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn counts_stdin_lines() {
    let output = cargo_bin_cmd!("wc")
        .arg("-l")
        .write_stdin("one\ntwo\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(output).unwrap().trim(), "2 <stdin>");
}
