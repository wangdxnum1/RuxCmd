use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn returns_failure() {
    cargo_bin_cmd!("false").assert().code(1);
}

#[test]
fn ignores_operands_and_unknown_options() {
    // POSIX: false ignores all operands and unknown options, still exits 1.
    cargo_bin_cmd!("false")
        .args(["--anything", "-x", "foo"])
        .assert()
        .code(1);
    cargo_bin_cmd!("false").arg("bar").assert().code(1);
}
