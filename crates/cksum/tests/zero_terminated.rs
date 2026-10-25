use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn zero_crc_output_keeps_posix_vector_and_stdin_naming() {
    for flag in ["-z", "--zero"] {
        cargo_bin_cmd!("cksum")
            .arg(flag)
            .write_stdin("hello")
            .assert()
            .success()
            .stdout(b"3287646509 5\0".to_vec());
        cargo_bin_cmd!("cksum")
            .args([flag, "-"])
            .write_stdin("hello")
            .assert()
            .success()
            .stdout(b"3287646509 5 -\0".to_vec());
        cargo_bin_cmd!("cksum")
            .args([flag, "-a", "crc"])
            .write_stdin("")
            .assert()
            .success()
            .stdout(b"4294967295 0\0".to_vec());
    }
}

#[test]
fn zero_crc_output_terminates_each_named_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("one"), b"hello").unwrap();
    std::fs::write(dir.path().join("two"), b"").unwrap();
    cargo_bin_cmd!("cksum")
        .current_dir(dir.path())
        .args(["-z", "one", "two"])
        .assert()
        .success()
        .stdout(
            b"3287646509 5 one\0"
                .iter()
                .chain(b"4294967295 0 two\0")
                .copied()
                .collect::<Vec<_>>(),
        );
}
