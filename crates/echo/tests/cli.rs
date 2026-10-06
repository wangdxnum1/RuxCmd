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
