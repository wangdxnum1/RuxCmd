use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn prints_clean_forward_slash_path_without_verbatim_prefix() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("prog.exe"), b"").unwrap();

    // Expected: the PATH-relative as-written path with forward slashes
    // (which prints the PATH entry verbatim, no canonicalization).
    let expected = dir
        .path()
        .join("prog.exe")
        .to_string_lossy()
        .replace('\\', "/");

    cargo_bin_cmd!("which")
        .env("PATH", dir.path())
        .arg("prog")
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}
