use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn prints_a_single_non_empty_line_and_exits_zero() {
    let output = cargo_bin_cmd!("logname")
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(
        stdout.ends_with('\n'),
        "output must end with a newline, got: {stdout:?}"
    );
    let name = stdout.trim_end_matches('\n');
    assert!(!name.is_empty(), "login name must not be empty");
    assert!(
        !name.contains('\n') && !name.contains('\r'),
        "exactly one line expected, got: {stdout:?}"
    );
}

#[test]
fn extra_operand_is_an_error() {
    cargo_bin_cmd!("logname")
        .arg("someone")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("extra operand"));
}
