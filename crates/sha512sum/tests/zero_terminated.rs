use assert_cmd::cargo::cargo_bin_cmd;

const HELLO: &str = "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043";
const EMPTY: &str = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";

#[test]
fn zero_output_disables_filename_escaping_and_supports_tags() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("nested")).unwrap();
    std::fs::write(dir.path().join("nested/file"), b"hello").unwrap();
    for flag in ["-z", "--zero"] {
        cargo_bin_cmd!("sha512sum")
            .current_dir(dir.path())
            .args([flag, r"nested\file"])
            .assert()
            .success()
            .stdout(format!("{HELLO}  nested\\file\0"));
        cargo_bin_cmd!("sha512sum")
            .current_dir(dir.path())
            .args([flag, "--tag", r"nested\file"])
            .assert()
            .success()
            .stdout(format!("SHA512 (nested\\file) = {HELLO}\0"));
        cargo_bin_cmd!("sha512sum")
            .arg(flag)
            .write_stdin("hello")
            .assert()
            .success()
            .stdout(format!("{HELLO}  -\0"));
        cargo_bin_cmd!("sha512sum")
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
        cargo_bin_cmd!("sha512sum")
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
        cargo_bin_cmd!("sha512sum")
            .current_dir(dir.path())
            .args(&args)
            .write_stdin(format!("{HELLO}  two  spaces\n"))
            .assert()
            .success()
            .stdout("two  spaces: OK\n");
        cargo_bin_cmd!("sha512sum")
            .args(&args)
            .write_stdin("")
            .assert()
            .code(1);
    }
}
