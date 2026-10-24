use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn root_path_prints_slash() {
    // Windows: Path::new("/").is_absolute() is false, but GNU dirname / -> /
    cargo_bin_cmd!("dirname")
        .arg("/")
        .assert()
        .success()
        .stdout("/\n");
    cargo_bin_cmd!("dirname")
        .arg("//")
        .assert()
        .success()
        .stdout("/\n");
}

#[test]
fn drive_root_prints_drive_with_slash() {
    cargo_bin_cmd!("dirname")
        .arg("C:/")
        .assert()
        .success()
        .stdout("C:/\n");
    cargo_bin_cmd!("dirname")
        .arg("C:/x")
        .assert()
        .success()
        .stdout("C:/\n");
}

#[test]
fn lone_drive_prefix_prints_dot() {
    cargo_bin_cmd!("dirname")
        .arg("C:")
        .assert()
        .success()
        .stdout(".\n");
}

#[test]
fn relative_and_regular_paths_unchanged() {
    cargo_bin_cmd!("dirname")
        .arg("a/b")
        .assert()
        .success()
        .stdout("a\n");
    cargo_bin_cmd!("dirname")
        .arg("a")
        .assert()
        .success()
        .stdout(".\n");
}
