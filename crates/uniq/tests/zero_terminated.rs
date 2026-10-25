use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_groups_keep_original_records_and_distinct_invalid_bytes() {
    for flag in ["-z", "--zero-terminated"] {
        cargo_bin_cmd!("uniq")
            .arg(flag)
            .write_stdin(b"a\nb\0a\nb\0\0\0\xff\0\xfe\0last".to_vec())
            .assert()
            .success()
            .stdout(b"a\nb\0\0\xff\0\xfe\0last\0".to_vec());
        cargo_bin_cmd!("uniq")
            .arg(flag)
            .write_stdin("")
            .assert()
            .success()
            .stdout("");
    }
}

#[test]
fn nul_count_case_duplicate_and_field_comparisons_work() {
    cargo_bin_cmd!("uniq")
        .args(["-z", "-c", "-i"])
        .write_stdin(b"A\0a\0B\0".to_vec())
        .assert()
        .success()
        .stdout(b"      2 A\0      1 B\0".to_vec());
    cargo_bin_cmd!("uniq")
        .args(["-z", "-d"])
        .write_stdin(b"a\0a\0b\0".to_vec())
        .assert()
        .success()
        .stdout(b"a\0".to_vec());
    cargo_bin_cmd!("uniq")
        .args(["-z", "-u"])
        .write_stdin(b"a\0a\0b\0".to_vec())
        .assert()
        .success()
        .stdout(b"b\0".to_vec());
    cargo_bin_cmd!("uniq")
        .args(["-z", "-f", "1"])
        .write_stdin(b"x a  b\0y a b\0z a\nb\0".to_vec())
        .assert()
        .success()
        .stdout(b"x a  b\0y a b\0z a\nb\0".to_vec());
}

#[test]
fn nul_skipped_fields_treat_lf_as_gnu_field_separator() {
    cargo_bin_cmd!("uniq")
        .args(["-z", "-f", "1"])
        .write_stdin(b"a\nb\0c\nb\0".to_vec())
        .assert()
        .success()
        .stdout(b"a\nb\0".to_vec());
    cargo_bin_cmd!("uniq")
        .args(["-z", "-f", "1"])
        .write_stdin(b"\xff\na\0\xfe\rb\0\0z\0\xc3(\0".to_vec())
        .assert()
        .success()
        .stdout(b"\xff\na\0\xfe\rb\0".to_vec());
}
