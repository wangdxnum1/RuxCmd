use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn returns_success() {
    cargo_bin_cmd!("true").assert().code(0);
}
