use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn explicit_delimiter_is_kept_in_output() {
    cargo_bin_cmd!("numfmt")
        .args(["-d", ",", "--to=si", "--field=2"])
        .write_stdin("a,4096\n")
        .assert()
        .success()
        .stdout("a,4.1K\n");
}

#[test]
fn explicit_delimiter_does_not_fold_consecutive_separators() {
    cargo_bin_cmd!("numfmt")
        .args(["-d", ",", "--to=si", "--field=3"])
        .write_stdin(",,1000\n")
        .assert()
        .success()
        .stdout(",,1.0K\n");
    cargo_bin_cmd!("numfmt")
        .args(["-d", ",", "--to=si"])
        .write_stdin("1,,2\n")
        .assert()
        .success()
        .stdout("1,,2\n");
}

#[test]
fn from_auto_uses_1000_for_k_and_1024_for_ki() {
    cargo_bin_cmd!("numfmt")
        .args(["--from=auto"])
        .write_stdin("1K\n")
        .assert()
        .success()
        .stdout("1000\n");
    cargo_bin_cmd!("numfmt")
        .args(["--from=auto"])
        .write_stdin("1Ki\n")
        .assert()
        .success()
        .stdout("1024\n");
    cargo_bin_cmd!("numfmt")
        .args(["--from=auto"])
        .write_stdin("1M\n")
        .assert()
        .success()
        .stdout("1000000\n");
    cargo_bin_cmd!("numfmt")
        .args(["--from=auto"])
        .write_stdin("1Mi\n")
        .assert()
        .success()
        .stdout("1048576\n");
}

#[test]
fn default_whitespace_mode_still_folds_and_joins_with_space() {
    cargo_bin_cmd!("numfmt")
        .args(["--to=si"])
        .write_stdin("  1000   2\n")
        .assert()
        .success()
        .stdout("1.0K 2\n");
}
