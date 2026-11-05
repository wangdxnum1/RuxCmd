//! Behaviour tests for the batch-B grep additions: `--include`/`--exclude`/
//! `--exclude-dir`, `-m`, `-z` and `-L`.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Path;

fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), "hit\nhit\nhit\n").unwrap();
    fs::write(dir.path().join("b.log"), "hit\n").unwrap();
    fs::write(dir.path().join("sub").join("c.txt"), "hit\n").unwrap();
    dir
}

fn stdout_of(dir: &Path, args: &[&str]) -> String {
    let assert = cargo_bin_cmd!("grep").args(args).current_dir(dir).assert();
    String::from_utf8_lossy(&assert.get_output().stdout).into_owned()
}

#[test]
fn include_limits_search_to_matching_names() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-r", "--include=*.log", "hit", "."]);
    assert_eq!(out.lines().count(), 1, "stdout: {:?}", out);
    assert!(out.contains("b.log"), "stdout: {:?}", out);
}

#[test]
fn include_applies_to_explicit_operands_like_gnu() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["--include=*.log", "hit", "a.txt", "b.log"]);
    assert_eq!(out.lines().count(), 1, "stdout: {:?}", out);
    assert!(out.contains("b.log"), "stdout: {:?}", out);
}

#[test]
fn exclude_skips_matching_names() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-r", "--exclude=*.txt", "hit", "."]);
    assert!(!out.contains("a.txt"), "stdout: {:?}", out);
    assert!(!out.contains("c.txt"), "stdout: {:?}", out);
    assert!(out.contains("b.log"), "stdout: {:?}", out);
}

#[test]
fn exclude_dir_skips_recursion() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-r", "--exclude-dir=sub", "hit", "."]);
    assert!(!out.contains("c.txt"), "stdout: {:?}", out);
    assert!(out.contains("a.txt"), "stdout: {:?}", out);
}

#[test]
fn max_count_stops_after_the_nth_match() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-m", "2", "hit", "a.txt"]);
    assert_eq!(out.lines().count(), 2, "stdout: {:?}", out);
}

#[test]
fn max_count_is_per_file_when_recursive() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-r", "-m", "1", "hit", "."]);
    // a.txt and b.log each contribute exactly one line.
    assert_eq!(out.lines().count(), 3, "stdout: {:?}", out);
}

#[test]
fn max_count_caps_the_count_output() {
    let dir = tree();
    let out = stdout_of(dir.path(), &["-m", "2", "-c", "hit", "a.txt"]);
    assert_eq!(out.trim(), "2");
}

#[test]
fn max_count_zero_matches_nothing() {
    let dir = tree();
    cargo_bin_cmd!("grep")
        .args(["-m", "0", "hit", "a.txt"])
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::is_empty());
}

#[test]
fn nul_records_are_read_and_written_with_nul() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("nul.txt"), b"miss\0hit\0miss\0").unwrap();

    let assert = cargo_bin_cmd!("grep")
        .args(["-z", "hit", "nul.txt"])
        .current_dir(dir.path())
        .assert()
        .success();
    assert_eq!(assert.get_output().stdout, b"hit\0");
}

#[test]
fn nul_records_count_as_single_records() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("nul.txt"), b"miss\0hit\0miss\0").unwrap();

    let out = stdout_of(dir.path(), &["-z", "-c", "hit", "nul.txt"]);
    assert_eq!(out.trim(), "1");
}

#[test]
fn files_without_match_lists_only_non_matching_files() {
    let dir = tree();
    let assert = cargo_bin_cmd!("grep")
        .args(["-L", "hit", "a.txt", "b.log"])
        .current_dir(dir.path())
        .assert();
    // Every file matches, so nothing is listed, but matches still set exit 0.
    assert!(assert.get_output().stdout.is_empty());
    assert!(assert.get_output().status.success());

    let assert = cargo_bin_cmd!("grep")
        .args(["-L", "absent", "a.txt", "b.log"])
        .current_dir(dir.path())
        .assert()
        .code(1);
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    assert_eq!(out.lines().count(), 2, "stdout: {:?}", out);
}

#[test]
fn the_last_of_l_and_big_l_wins() {
    let dir = tree();
    // -l wins here: both files have a match.
    let out = stdout_of(dir.path(), &["-L", "-l", "hit", "a.txt", "b.log"]);
    assert_eq!(out.lines().count(), 2, "stdout: {:?}", out);

    // -L wins here: neither file is listed because both match.
    let out = stdout_of(dir.path(), &["-l", "-L", "hit", "a.txt", "b.log"]);
    assert!(out.is_empty(), "stdout: {:?}", out);
}
