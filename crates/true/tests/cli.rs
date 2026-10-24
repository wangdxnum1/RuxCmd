use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn returns_success() {
    cargo_bin_cmd!("true").assert().code(0);
}

#[test]
fn ignores_operands_and_unknown_options() {
    // POSIX: true ignores all operands and unknown options.
    cargo_bin_cmd!("true")
        .args(["--anything", "-x", "foo"])
        .assert()
        .code(0);
    cargo_bin_cmd!("true").arg("bar").assert().code(0);
}
