use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Component;
use tempfile::TempDir;

#[test]
fn default_output_is_clean_absolute_path() {
    let dir = TempDir::new().unwrap();

    let canon = fs::canonicalize(dir.path()).unwrap();
    let s = canon.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    let expected = s.replace('\\', "/");
    assert!(!expected.contains(r"\\?\"));

    cargo_bin_cmd!("realpath")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}

#[test]
fn strip_components_counts_only_name_components() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("x").join("y");
    fs::create_dir_all(&target).unwrap();

    // Prefix (drive) and RootDir are structural: --strip-components=1 must
    // strip the first *name* component only, leaving a relative a/b-style
    // remainder instead of the old off-by-one leading "/".
    let canon = fs::canonicalize(&target).unwrap();
    let names: Vec<String> = canon
        .components()
        .filter_map(|c| match c {
            Component::Normal(n) => Some(n.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    assert!(names.len() >= 2);
    let expected = names[1..].join("/");
    assert!(!expected.starts_with('/'));

    cargo_bin_cmd!("realpath")
        .args(["--strip-components=1"])
        .arg(&target)
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}
