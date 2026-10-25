use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn removes_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("empty");
    fs::create_dir(&sub).unwrap();

    cargo_bin_cmd!("rmdir")
        .arg(&sub)
        .assert()
        .success()
        .code(0)
        .stdout("")
        .stderr("");
    assert!(!sub.exists(), "the empty directory must be gone");
}

#[test]
fn fails_on_non_empty_directory_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("full");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("keep.txt"), "data").unwrap();

    cargo_bin_cmd!("rmdir")
        .arg(&sub)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("failed to remove"));
    assert!(sub.is_dir(), "the non-empty directory must survive");
}

#[test]
fn ignore_fail_on_non_empty_exits_zero_and_keeps_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("full");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("keep.txt"), "data").unwrap();

    cargo_bin_cmd!("rmdir")
        .arg("--ignore-fail-on-non-empty")
        .arg(&sub)
        .assert()
        .success()
        .code(0)
        .stderr("");
    assert!(sub.is_dir(), "the non-empty directory must survive");
}

#[test]
fn ignore_flag_does_not_hide_other_failures() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-dir");

    // A missing directory is not a non-empty-directory failure.
    cargo_bin_cmd!("rmdir")
        .arg("--ignore-fail-on-non-empty")
        .arg(&missing)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("failed to remove"));
}

#[test]
fn parents_removes_chain_of_empty_directories() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();

    // Relative path: the climb stops at the topmost component, so the
    // sandbox root itself is never touched.
    cargo_bin_cmd!("rmdir")
        .arg("-p")
        .arg("a/b/c")
        .current_dir(dir.path())
        .assert()
        .success()
        .code(0);
    assert!(!dir.path().join("a").exists(), "whole chain must be gone");
}

#[test]
fn parents_stops_at_non_empty_ancestor_with_error() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::write(dir.path().join("a/keep.txt"), "data").unwrap();

    cargo_bin_cmd!("rmdir")
        .arg("-p")
        .arg("a/b/c")
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("failed to remove directory"));
    assert!(dir.path().join("a").is_dir(), "'a' must survive");
    assert!(
        dir.path().join("a/keep.txt").exists(),
        "'a' contents must survive"
    );
    assert!(
        !dir.path().join("a/b").exists(),
        "removed chain below 'a' must be gone"
    );
}

#[test]
fn verbose_prints_removal_line_for_each_directory() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("x/y")).unwrap();

    cargo_bin_cmd!("rmdir")
        .args(["-p", "-v", "x/y"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(format!(
            "rmdir: removing directory, 'x/y'\nrmdir: removing directory, 'x'\n"
        ));
    assert!(!dir.path().join("x").exists());
}

#[test]
fn missing_operand_errors() {
    cargo_bin_cmd!("rmdir")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("rmdir: missing operand"));
}
