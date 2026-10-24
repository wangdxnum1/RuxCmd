use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn default_output_is_procps_format() {
    let out = cargo_bin_cmd!("uptime").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(
        stdout.contains(" up ") && stdout.trim_end().ends_with(", 0 users"),
        "stdout: {:?}",
        stdout
    );
}
