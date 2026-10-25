use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn directory_flag_creates_multi_level_directories() {
    let dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("install")
        .args(["-d", "a/b/c"])
        .current_dir(dir.path())
        .assert()
        .success()
        .code(0);
    assert!(dir.path().join("a/b/c").is_dir(), "a/b/c must be created");
}

#[test]
fn single_source_copies_to_file_name() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "payload").unwrap();
    let dest = dir.path().join("dest.txt");

    cargo_bin_cmd!("install")
        .arg(&src)
        .arg(&dest)
        .assert()
        .success()
        .code(0);
    assert_eq!(fs::read(&dest).unwrap(), b"payload");
}

#[test]
fn single_source_into_existing_directory_uses_source_name() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("app.exe");
    fs::write(&src, "bin").unwrap();
    let dest_dir = dir.path().join("bin");
    fs::create_dir(&dest_dir).unwrap();

    cargo_bin_cmd!("install")
        .arg(&src)
        .arg(&dest_dir)
        .assert()
        .success()
        .code(0);
    assert!(dest_dir.join("app.exe").is_file());
    assert_eq!(fs::read(dest_dir.join("app.exe")).unwrap(), b"bin");
}

#[test]
fn multiple_sources_require_existing_directory() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();
    let dest_dir = dir.path().join("out");
    fs::create_dir(&dest_dir).unwrap();

    cargo_bin_cmd!("install")
        .arg(&a)
        .arg(&b)
        .arg(&dest_dir)
        .assert()
        .success()
        .code(0);
    assert_eq!(fs::read(dest_dir.join("a.txt")).unwrap(), b"A");
    assert_eq!(fs::read(dest_dir.join("b.txt")).unwrap(), b"B");

    // With several sources a plain file destination is rejected (GNU).
    let not_dir = dir.path().join("plain.txt");
    fs::write(&not_dir, "x").unwrap();
    cargo_bin_cmd!("install")
        .arg(&a)
        .arg(&b)
        .arg(&not_dir)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("not a directory"));
}

#[test]
fn mode_000_makes_destination_readonly() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("ro.txt");
    fs::write(&src, "data").unwrap();
    let dest = dir.path().join("ro-dest.txt");

    cargo_bin_cmd!("install")
        .args(["-m", "000"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert!(
        fs::metadata(&dest).unwrap().permissions().readonly(),
        "-m 000 must map onto the read-only attribute"
    );
}

#[test]
fn mode_755_keeps_destination_writable() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("rw.txt");
    fs::write(&src, "data").unwrap();
    let dest = dir.path().join("rw-dest.txt");

    cargo_bin_cmd!("install")
        .args(["-m", "755"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert!(
        !fs::metadata(&dest).unwrap().permissions().readonly(),
        "-m 755 must keep the destination writable"
    );
}

#[test]
fn invalid_mode_errors() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("s.txt");
    fs::write(&src, "d").unwrap();

    cargo_bin_cmd!("install")
        .args(["-m", "abc"])
        .arg(&src)
        .arg(dir.path().join("d.txt"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("invalid mode"));
}

#[test]
fn verbose_prints_destination_line() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("in.txt");
    fs::write(&src, "v").unwrap();

    cargo_bin_cmd!("install")
        .args(["-v", "in.txt", "out.txt"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("install -> 'out.txt'\n");
}

#[test]
fn missing_operand_errors() {
    cargo_bin_cmd!("install")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("missing file operand"));
}
