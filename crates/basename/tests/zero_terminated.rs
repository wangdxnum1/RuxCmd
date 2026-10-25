use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn zero_output_preserves_newlines_and_existing_suffix_rules() {
    for flag in ["-z", "--zero"] {
        cargo_bin_cmd!("basename")
            .args([flag, "-a", "dir/a\nx.txt", "dir/b.txt"])
            .assert()
            .success()
            .stdout(b"a\nx.txt\0b.txt\0".to_vec());
        cargo_bin_cmd!("basename")
            .args([flag, "dir/a.txt", ".txt"])
            .assert()
            .success()
            .stdout(b"a\0".to_vec());
    }
}
