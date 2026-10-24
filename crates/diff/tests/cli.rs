use assert_cmd::cargo::cargo_bin_cmd;

fn write_file(dir: &tempfile::TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn identical_files_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "a.txt", "a\nb\nc\n");
    let f2 = write_file(&dir, "b.txt", "a\nb\nc\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(0)
        .stdout("");
}

#[test]
fn split_delete_add_uses_gnu_line_numbers() {
    // GNU normal-format numbering for the a,b,x vs a,b,y pair: deleting
    // file1 line 3 anchors before file2 line 3 ("3d2"), adding file2 line 3
    // anchors after file1 line 2 ("2a3"). GNU merges the pair into a single
    // "3c3" hunk; RuxCmd keeps the per-command split (documented deviation),
    // but each command must carry correct line numbers.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nx\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\ny\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("3d2\n< x\n2a3\n> y\n");
}

#[test]
fn differing_files_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "one\n");
    let f2 = write_file(&dir, "f2.txt", "two\n");

    cargo_bin_cmd!("diff").arg(&f1).arg(&f2).assert().code(1);
}

#[test]
fn missing_file_exits_two_with_error_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "one\n");
    let missing = dir.path().join("missing.txt");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&missing)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("Error reading"));
}

#[test]
fn unified_format_exits_one_on_difference() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nx\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\ny\n");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(predicates::str::contains("@@"));
}
