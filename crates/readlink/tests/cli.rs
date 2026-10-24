use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

#[test]
fn canonicalize_outputs_clean_forward_slash_path() {
    let dir = TempDir::new().unwrap();

    // -f resolves every component to a canonical path, which on Windows is a
    // \\?\ verbatim path; the output must be cleaned up instead of leaked.
    let canon = fs::canonicalize(dir.path()).unwrap();
    let s = canon.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    let expected = s.replace('\\', "/");
    assert!(!expected.contains(r"\\?\"));

    cargo_bin_cmd!("readlink")
        .arg("-f")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}
