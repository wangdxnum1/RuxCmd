use assert_cmd::cargo::cargo_bin_cmd;

const HELLO: &str = "5d41402abc4b2a76b9719d911017c592";
const EMPTY: &str = "d41d8cd98f00b204e9800998ecf8427e";

#[test]
fn zero_output_disables_filename_escaping_and_supports_tags() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("nested")).unwrap();
    std::fs::write(dir.path().join("nested/file"), b"hello").unwrap();
    for flag in ["-z", "--zero"] {
        cargo_bin_cmd!("md5sum")
            .current_dir(dir.path())
            .args([flag, r"nested\file"])
            .assert()
            .success()
            .stdout(format!("{HELLO}  nested\\file\0"));
        cargo_bin_cmd!("md5sum")
            .current_dir(dir.path())
            .args([flag, "--tag", r"nested\file"])
            .assert()
            .success()
            .stdout(format!("MD5 (nested\\file) = {HELLO}\0"));
        cargo_bin_cmd!("md5sum")
            .arg(flag)
            .write_stdin("hello")
            .assert()
            .success()
            .stdout(format!("{HELLO}  -\0"));
        cargo_bin_cmd!("md5sum")
            .args([flag, "-"])
            .write_stdin("")
            .assert()
            .success()
            .stdout(format!("{EMPTY}  -\0"));
    }
}

#[test]
fn zero_cannot_be_used_for_verification() {
    for args in [["-z", "-c"], ["--check", "--zero"]] {
        cargo_bin_cmd!("md5sum")
            .args(args)
            .write_stdin("")
            .assert()
            .code(1)
            .stdout("");
    }
}

#[test]
fn normal_check_mode_reads_stdin_and_preserves_filename_spaces() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("two  spaces"), b"hello").unwrap();
    for args in [vec!["-c"], vec!["-c", "-"]] {
        cargo_bin_cmd!("md5sum")
            .current_dir(dir.path())
            .args(&args)
            .write_stdin(format!("{HELLO}  two  spaces\n"))
            .assert()
            .success()
            .stdout("two  spaces: OK\n");
        cargo_bin_cmd!("md5sum")
            .args(&args)
            .write_stdin("")
            .assert()
            .code(1);
    }
}
