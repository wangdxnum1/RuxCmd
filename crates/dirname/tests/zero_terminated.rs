use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn zero_output_terminates_each_operand_without_rewriting_content() {
    for flag in ["-z", "--zero"] {
        cargo_bin_cmd!("dirname")
            .args([flag, "one\nline/file", "a/b.txt", "/"])
            .assert()
            .success()
            .stdout(b"one\nline\0a\0/\0".to_vec());
    }
}
