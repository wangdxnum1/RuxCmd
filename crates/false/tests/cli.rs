use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn returns_failure() {
    cargo_bin_cmd!("false").assert().code(1);
}
