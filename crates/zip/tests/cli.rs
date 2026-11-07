use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

fn entry_names(archive: &Path) -> Vec<String> {
    let file = File::open(archive).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect()
}

fn entry_content(archive: &Path, name: &str) -> Vec<u8> {
    let file = File::open(archive).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut entry = zip.by_name(name).unwrap();
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn archives_a_single_file() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "hello").unwrap();

    cargo_bin_cmd!("zip")
        .arg("out.zip")
        .arg("a.txt")
        .current_dir(tmp.path())
        .assert()
        .success();

    assert_eq!(entry_names(&tmp.path().join("out.zip")), ["a.txt"]);
    assert_eq!(
        entry_content(&tmp.path().join("out.zip"), "a.txt"),
        b"hello"
    );
}

#[test]
fn quiet_mode_suppresses_per_entry_output() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "hello").unwrap();

    cargo_bin_cmd!("zip")
        .arg("-q")
        .arg("out.zip")
        .arg("a.txt")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

#[test]
fn recurse_includes_nested_entries() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("dir")).unwrap();
    fs::create_dir(tmp.path().join("dir").join("sub")).unwrap();
    fs::write(tmp.path().join("dir").join("sub").join("x.txt"), "x").unwrap();

    cargo_bin_cmd!("zip")
        .args(["-q", "-r", "out.zip", "dir"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let mut names = entry_names(&tmp.path().join("out.zip"));
    names.sort();
    assert_eq!(names, ["dir/", "dir/sub/", "dir/sub/x.txt"]);
    assert_eq!(
        entry_content(&tmp.path().join("out.zip"), "dir/sub/x.txt"),
        b"x"
    );
}

#[test]
fn without_recurse_only_the_directory_entry_is_added() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir(tmp.path().join("dir")).unwrap();
    fs::write(tmp.path().join("dir").join("x.txt"), "x").unwrap();

    cargo_bin_cmd!("zip")
        .arg("-q")
        .arg("out.zip")
        .arg("dir")
        .current_dir(tmp.path())
        .assert()
        .success();

    assert_eq!(entry_names(&tmp.path().join("out.zip")), ["dir/"]);
}

#[test]
fn missing_input_fails_with_a_message() {
    let tmp = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("zip")
        .arg("out.zip")
        .arg("nope.txt")
        .current_dir(tmp.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains("zip:"));

    // The archive is written atomically via a temp file: a failed run must
    // not leave any archive behind at all.
    assert!(!tmp.path().join("out.zip").exists());
}

#[test]
fn unsupported_options_are_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "hello").unwrap();
    cargo_bin_cmd!("zip")
        .args(["-x", "out.zip", "a.txt"])
        .current_dir(tmp.path())
        .assert()
        .code(2)
        .stderr(predicates::str::contains("unexpected argument"));
}
