use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn joins_arguments_and_supports_no_newline() {
    cargo_bin_cmd!("echo")
        .args(["hello", "world"])
        .assert()
        .success()
        .stdout("hello world\n");
    cargo_bin_cmd!("echo")
        .args(["-n", "hello"])
        .assert()
        .success()
        .stdout("hello");
}

#[test]
fn default_does_not_interpret_escapes() {
    // GNU echo defaults to -E: backslashes stay literal, so Windows paths
    // such as C:\new must survive unchanged.
    cargo_bin_cmd!("echo")
        .args(["C:\\new"])
        .assert()
        .success()
        .stdout("C:\\new\n");
}

#[test]
fn e_flag_enables_escape_interpretation() {
    cargo_bin_cmd!("echo")
        .args(["-e", "a\\nb"])
        .assert()
        .success()
        .stdout("a\nb\n");
}

#[test]
fn capital_e_flag_disables_escape_interpretation() {
    cargo_bin_cmd!("echo")
        .args(["-E", "a\\nb"])
        .assert()
        .success()
        .stdout("a\\nb\n");
}
