use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn prints_clean_forward_slash_path_without_verbatim_prefix() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("prog.exe"), b"").unwrap();

    // Expected: the canonical path with the \\?\ verbatim prefix stripped
    // and backslashes normalized to forward slashes.
    let canon = fs::canonicalize(dir.path().join("prog.exe")).unwrap();
    let s = canon.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    let expected = s.replace('\\', "/");
    assert!(!expected.contains(r"\\?\"));

    cargo_bin_cmd!("which")
        .env("PATH", dir.path())
        .arg("prog")
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}
