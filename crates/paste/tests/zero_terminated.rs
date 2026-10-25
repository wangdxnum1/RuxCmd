use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_parallel_operands_share_stdin_and_preserve_record_bytes() {
    for flag in ["-z", "--zero-terminated"] {
        cargo_bin_cmd!("paste")
            .args([flag, "-", "-"])
            .write_stdin(b"a\nx\0b\r\0c\0".to_vec())
            .assert()
            .success()
            .stdout(b"a\nx\tb\r\0c\t\0".to_vec());
    }
}

#[test]
fn nul_serial_mode_handles_empty_input_and_delimiter_escapes() {
    cargo_bin_cmd!("paste")
        .args(["-z", "-s"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(b"\0".to_vec());
    for (delimiter, expected) in [
        (r"\n", b"a\nb\0".as_slice()),
        (r"\0", b"ab\0"),
        (r"\\", b"a\\b\0"),
    ] {
        cargo_bin_cmd!("paste")
            .args(["-z", "-s", "-d", delimiter])
            .write_stdin(b"a\0b".to_vec())
            .assert()
            .success()
            .stdout(expected.to_vec());
    }
    cargo_bin_cmd!("paste")
        .args(["-z", "-s"])
        .write_stdin(b"\xff\0\xfe\0".to_vec())
        .assert()
        .success()
        .stdout(b"\xff\t\xfe\0".to_vec());
}
