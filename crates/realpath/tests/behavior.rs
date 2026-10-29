use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn relative_existing_path_resolves_absolute() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    let out = cargo_bin_cmd!("realpath")
        .arg("sub")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // paths::display normalizes separators to forward slashes.
    let expected = dir.path().join("sub").to_string_lossy().replace("\\", "/");
    assert_eq!(text, expected, "realpath must print the canonical path");
}

#[test]
fn nonexistent_path_fails_with_exit_one() {
    let dir = TempDir::new().unwrap();
    let out = cargo_bin_cmd!("realpath")
        .arg("no-such-thing")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("realpath:"), "stderr: {err}");
}

#[test]
fn strip_components_removes_leading_names() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("x/y")).unwrap();
    let out = cargo_bin_cmd!("realpath")
        .args(["--strip-components", "2"])
        .arg("x/y")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // GNU strips components from the canonicalized absolute path; rebuild
    // the expectation from the temp dir's own components.
    let full = fs::canonicalize(dir.path()).unwrap();
    let names: Vec<String> = full
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(n) => Some(n.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let expected = format!("{}/x/y", names[2..].join("/"));
    assert_eq!(text, expected, "stripped path: {text}");
}

#[test]
fn dot_segments_are_resolved() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("inner")).unwrap();
    let out = cargo_bin_cmd!("realpath")
        .arg("inner/../inner")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let expected = dir
        .path()
        .join("inner")
        .to_string_lossy()
        .replace("\\", "/");
    assert_eq!(text, expected);
}
