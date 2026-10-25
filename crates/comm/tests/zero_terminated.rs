use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_comparison_keeps_bytes_and_compacts_hidden_columns() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("one");
    let second = dir.path().join("two");
    std::fs::write(&first, b"\0a\nx\0c\0\xff\0").unwrap();
    std::fs::write(&second, b"\0b\r\0c\0\xfe\0").unwrap();
    for flag in ["-z", "--zero-terminated"] {
        cargo_bin_cmd!("comm")
            .arg(flag)
            .arg(&first)
            .arg(&second)
            .assert()
            .success()
            .stdout(b"\t\t\0a\nx\0\tb\r\0\t\tc\0\t\xfe\0\xff\0".to_vec());
        cargo_bin_cmd!("comm")
            .args([flag, "-1", "-2"])
            .arg(&first)
            .arg(&second)
            .assert()
            .success()
            .stdout(b"\0c\0".to_vec());
    }
}
