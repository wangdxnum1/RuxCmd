use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_shuffle_preserves_record_multiset_and_seed_reproducibility() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed");
    std::fs::write(&seed, b"fixed source bytes").unwrap();
    let input = b"a\nb\0\0\xff\0\xfe\0a\nb";
    let mut outputs = Vec::new();
    for flag in ["-z", "--zero-terminated"] {
        let result = cargo_bin_cmd!("shuf")
            .arg(flag)
            .arg("--random-source")
            .arg(&seed)
            .write_stdin(input.to_vec())
            .assert()
            .success();
        let output = result.get_output().stdout.clone();
        assert_eq!(output.last(), Some(&0));
        let mut records: Vec<_> = output[..output.len() - 1].split(|b| *b == 0).collect();
        records.sort();
        let mut expected: Vec<_> = input.split(|b| *b == 0).collect();
        expected.sort();
        assert_eq!(records, expected);
        outputs.push(output);
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn nul_range_count_and_output_file_work() {
    cargo_bin_cmd!("shuf")
        .args(["-z", "-i", "7-7"])
        .assert()
        .success()
        .stdout(b"7\0".to_vec());
    cargo_bin_cmd!("shuf")
        .args(["-z", "-n", "0"])
        .write_stdin(b"a\0b\0".to_vec())
        .assert()
        .success()
        .stdout("");
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("result");
    cargo_bin_cmd!("shuf")
        .args(["-z", "-i", "7-7", "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");
    assert_eq!(std::fs::read(out).unwrap(), b"7\0");
}
