use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn prints_root_and_entries() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), b"").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    let out = cargo_bin_cmd!("tree")
        .arg(dir.path().to_string_lossy().into_owned())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("a.txt"), "output: {text}");
    assert!(text.contains("sub"), "output: {text}");
    assert!(text.contains("directories"), "output: {text}");
}

#[test]
fn level_limits_depth() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    let out = cargo_bin_cmd!("tree")
        .args(["-L", "1"])
        .arg(dir.path().to_string_lossy().into_owned())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains('a'), "output: {text}");
    assert!(
        !text.contains("── b"),
        "depth-limited output should not contain b: {text}"
    );
}

#[test]
fn ascii_charset() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f.txt"), b"").unwrap();
    let out = cargo_bin_cmd!("tree")
        .args(["--charset=ascii"])
        .arg(dir.path().to_string_lossy().into_owned())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("`--"), "output: {text}");
}

#[test]
fn missing_dir_fails() {
    cargo_bin_cmd!("tree")
        .arg("definitely-missing-dir-xyz")
        .assert()
        .failure()
        .code(1);
}
