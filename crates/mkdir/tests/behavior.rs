use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn run(dir: &TempDir, args: &[&str]) -> (String, String, Option<i32>) {
    let out = cargo_bin_cmd!("mkdir")
        .args(args)
        .current_dir(dir.path())
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn creates_a_directory_silently() {
    let dir = TempDir::new().unwrap();
    let (out, err, code) = run(&dir, &["new"]);
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.is_empty());
    assert!(dir.path().join("new").is_dir());
}

#[test]
fn existing_directory_fails_without_parents() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("exists")).unwrap();
    let (_out, err, code) = run(&dir, &["exists"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("mkdir: cannot create directory 'exists'"),
        "stderr: {err}"
    );
}

#[test]
fn parents_creates_nested_directories() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&dir, &["-p", "a/b/c"]);
    assert_eq!(code, Some(0));
    assert!(err.is_empty());
    assert!(dir.path().join("a/b/c").is_dir());
}

#[test]
fn parents_are_idempotent() {
    let dir = TempDir::new().unwrap();
    let (_o, _e, c1) = run(&dir, &["-p", "a/b"]);
    let (_o, _e, c2) = run(&dir, &["-p", "a/b"]);
    assert_eq!(c1, Some(0));
    assert_eq!(c2, Some(0));
}

#[test]
fn missing_parent_fails_without_parents() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&dir, &["a/b"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("mkdir:"), "stderr: {err}");
}

#[test]
fn verbose_prints_gnu_style_message() {
    let dir = TempDir::new().unwrap();
    let (out, _err, code) = run(&dir, &["-v", "made"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, "mkdir: created directory 'made'\n");
}

#[test]
fn mode_option_is_accepted() {
    // -m parses but has no Windows equivalent; it must not fail.
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&dir, &["-m", "444", "restricted"]);
    assert_eq!(code, Some(0));
    assert!(err.is_empty());
}

#[test]
fn missing_operand_reports_error() {
    let out = cargo_bin_cmd!("mkdir").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("mkdir: missing operand"));
}
