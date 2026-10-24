//! Tests for the P1 fixes: link targets must be stored exactly as the user
//! wrote them (no `\\?\` verbatim canonicalization), and `-v` must be
//! --verbose (GNU semantics), not a version alias.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

/// Creating symbolic links on Windows requires privilege (or Developer
/// Mode); when the OS refuses with ERROR_PRIVILEGE_NOT_HELD the test skips.
fn skip_if_no_symlink_privilege(status: &std::process::ExitStatus, stderr: &str) -> bool {
    if status.success() {
        return false;
    }
    if stderr.contains("1314") || stderr.to_lowercase().contains("privilege") {
        eprintln!("skipped: symlink privilege not available on this host");
        return true;
    }
    false
}

#[test]
fn symlink_target_stored_as_written_relative() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("actual.txt"), "data").unwrap();
    let link = tmp.path().join("link.txt");

    let out = cargo_bin_cmd!("ln")
        .args(["-s", "actual.txt"])
        .arg(&link)
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    if skip_if_no_symlink_privilege(&out.status, &stderr) {
        return;
    }
    assert!(out.status.success(), "ln -s failed: {stderr}");

    let stored = fs::read_link(&link).unwrap();
    assert_eq!(
        stored.to_string_lossy(),
        "actual.txt",
        "target must be stored exactly as written"
    );
}

#[test]
fn symlink_target_stored_as_written_absolute() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("actual.txt");
    fs::write(&target, "data").unwrap();
    let link = tmp.path().join("link.txt");

    let out = cargo_bin_cmd!("ln")
        .arg("-s")
        .arg(&target)
        .arg(&link)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    if skip_if_no_symlink_privilege(&out.status, &stderr) {
        return;
    }
    assert!(out.status.success(), "ln -s failed: {stderr}");

    let stored = fs::read_link(&link).unwrap();
    assert!(
        !stored.as_os_str().to_string_lossy().starts_with(r"\\?\"),
        "stored target must not carry a verbatim prefix: {}",
        stored.display()
    );
    assert_eq!(stored, target, "target must equal the user-written path");
}

#[test]
fn verbose_flag_creates_link_and_prints_created() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("h.txt");
    fs::write(&target, "data").unwrap();
    let link = tmp.path().join("h2.txt");

    // -v is --verbose (hard links need no privilege).
    cargo_bin_cmd!("ln")
        .arg("-v")
        .arg(&target)
        .arg(&link)
        .assert()
        .success()
        .stdout(predicates::str::contains("created"));
    assert!(link.exists());

    // --version keeps printing the version banner.
    cargo_bin_cmd!("ln")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("ln 0."));
}
