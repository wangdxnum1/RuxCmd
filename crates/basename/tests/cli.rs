use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn trailing_dot_and_dotdot_components() {
    cargo_bin_cmd!("basename").arg("./").assert().success().stdout(".\n");
    cargo_bin_cmd!("basename").arg(".").assert().success().stdout(".\n");
    cargo_bin_cmd!("basename").arg("a/..").assert().success().stdout("..\n");
    cargo_bin_cmd!("basename").arg("..").assert().success().stdout("..\n");
}

#[test]
fn root_paths() {
    cargo_bin_cmd!("basename").arg("/").assert().success().stdout("/\n");
    cargo_bin_cmd!("basename").arg("//").assert().success().stdout("/\n");
    // GNU strips trailing separators, so the drive root's basename is "C:"
    cargo_bin_cmd!("basename").arg("C:/").assert().success().stdout("C:\n");
}

#[test]
fn regular_names_and_suffix_unchanged() {
    cargo_bin_cmd!("basename").arg("a/b").assert().success().stdout("b\n");
    cargo_bin_cmd!("basename")
        .args(["foo.c", "-s", ".c"])
        .assert()
        .success()
        .stdout("foo\n");
}
