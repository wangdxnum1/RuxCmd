use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_records_preserve_all_bytes_and_sort_distinct_invalid_bytes() {
    for flag in ["-z", "--zero-terminated"] {
        cargo_bin_cmd!("sort")
            .arg(flag)
            .write_stdin(b"b\nz\0a\r\0\0\xff\0\xfe\0last".to_vec())
            .assert()
            .success()
            .stdout(b"\0a\r\0b\nz\0last\0\xfe\0\xff\0".to_vec());
        cargo_bin_cmd!("sort")
            .arg(flag)
            .write_stdin("")
            .assert()
            .success()
            .stdout("");
    }
}

#[test]
fn nul_numeric_keys_unique_reverse_and_check_work() {
    cargo_bin_cmd!("sort")
        .args(["-z", "-n", "-u", "-t", ":", "-k", "2,2"])
        .write_stdin(b"a:20\nkept\0b:3\0c:3\0".to_vec())
        .assert()
        .success()
        .stdout(b"b:3\0a:20\nkept\0".to_vec());
    cargo_bin_cmd!("sort")
        .args(["-z", "-r", "-c"])
        .write_stdin(b"z\0m\0a".to_vec())
        .assert()
        .success()
        .stdout("");
    cargo_bin_cmd!("sort")
        .args(["-z", "-c"])
        .write_stdin(b"z\0a".to_vec())
        .assert()
        .code(1);
    cargo_bin_cmd!("sort")
        .args(["-z", "-u"])
        .write_stdin(b"\xff\0\xfe\0\xff\0".to_vec())
        .assert()
        .success()
        .stdout(b"\xfe\0\xff\0".to_vec());
}

#[test]
fn nul_fields_include_lf_and_numeric_keys_skip_it() {
    for unique in [false, true] {
        let mut command = cargo_bin_cmd!("sort");
        command.args(["-z", "-k", "2,2"]);
        if unique {
            command.arg("-u");
        }
        command
            .write_stdin(b"z\na\0a\nb\0".to_vec())
            .assert()
            .success()
            .stdout(b"z\na\0a\nb\0".to_vec());
    }
    for mode in ["-n", "-h"] {
        cargo_bin_cmd!("sort")
            .args(["-z", mode])
            .write_stdin(b"\n10\0\n2\0".to_vec())
            .assert()
            .success()
            .stdout(b"\n2\0\n10\0".to_vec());
    }
}
