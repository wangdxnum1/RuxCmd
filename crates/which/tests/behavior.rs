use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// Build a PATH sandbox: only the temp dir is on PATH so lookups are fully
/// controlled. Paths print through paths::display (forward slashes), so
/// expectations are built in the same form.
fn sandbox() -> TempDir {
    TempDir::new().unwrap()
}

fn to_display_form(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn run(dir: &TempDir, pathext: Option<&str>, args: &[&str]) -> (String, Option<i32>) {
    let mut cmd = cargo_bin_cmd!("which");
    if let Some(px) = pathext {
        cmd.env("PATHEXT", px);
    }
    let out = cmd.env("PATH", dir.path()).args(args).output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code(),
    )
}

#[test]
fn finds_executable_in_path_directory() {
    let dir = sandbox();
    fs::write(dir.path().join("foo.exe"), b"MZ").unwrap();
    let (out, code) = run(&dir, None, &["foo"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out,
        format!("{}\n", to_display_form(&dir.path().join("foo.exe")))
    );
}

#[test]
fn exact_name_is_tried_before_extensions() {
    let dir = sandbox();
    fs::write(dir.path().join("plain"), b"x").unwrap();
    fs::write(dir.path().join("plain.exe"), b"MZ").unwrap();
    let (out, code) = run(&dir, None, &["plain"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out,
        format!("{}\n", to_display_form(&dir.path().join("plain")))
    );
}

#[test]
fn all_lists_every_variant_in_probing_order() {
    let dir = sandbox();
    for name in ["bar.cmd", "bar.bat", "bar.exe", "bar.com"] {
        fs::write(dir.path().join(name), b"MZ").unwrap();
    }
    let (out, code) = run(&dir, None, &["--all", "bar"]);
    assert_eq!(code, Some(0));
    let lines: Vec<&str> = out.trim_end().lines().collect();
    let expected = ["bar.com", "bar.exe", "bar.bat", "bar.cmd"]
        .iter()
        .map(|n| to_display_form(&dir.path().join(n)))
        .collect::<Vec<_>>();
    assert_eq!(lines, expected);
}

#[test]
fn pathext_adds_custom_extensions() {
    let dir = sandbox();
    fs::write(dir.path().join("tool.xyz"), b"x").unwrap();

    let (_out, code) = run(&dir, Some(".XYZ"), &["tool"]);
    assert_eq!(code, Some(0));

    let (_out, code) = run(&dir, None, &["tool"]);
    assert_eq!(code, Some(1));
}

#[test]
fn not_found_exits_one_with_no_output() {
    let dir = sandbox();
    let (out, code) = run(&dir, None, &["definitely-not-here"]);
    assert_eq!(code, Some(1));
    assert!(out.is_empty());
}

#[test]
fn no_arguments_is_an_error() {
    let out = cargo_bin_cmd!("which").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("which:"));
}
