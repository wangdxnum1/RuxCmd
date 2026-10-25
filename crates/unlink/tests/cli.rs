use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn removes_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("gone.txt");
    fs::write(&target, "data").unwrap();

    cargo_bin_cmd!("unlink")
        .arg(&target)
        .assert()
        .success()
        .code(0)
        .stdout("")
        .stderr("");
    assert!(!target.exists(), "the file must be gone");
}

#[test]
fn removes_a_symbolic_link_itself() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.txt");
    let link = dir.path().join("alias.txt");
    fs::write(&real, "data").unwrap();

    // Creating symbolic links on Windows requires privilege (or Developer
    // Mode); when the OS refuses with ERROR_PRIVILEGE_NOT_HELD, skip.
    #[cfg(windows)]
    if let Err(e) = std::os::windows::fs::symlink_file(&real, &link) {
        if e.raw_os_error() == Some(1314) || e.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!("skipped: symlink privilege not available on this host");
            return;
        }
        panic!("symlink setup failed: {e}");
    }
    #[cfg(not(windows))]
    std::os::unix::fs::symlink(&real, &link).unwrap();

    cargo_bin_cmd!("unlink").arg(&link).assert().success();
    assert!(!link.exists(), "the symlink must be gone");
    assert!(real.exists(), "the pointed-to file must survive");
}

#[test]
fn missing_file_fails_with_gnu_message() {
    let dir = tempfile::tempdir().unwrap();

    cargo_bin_cmd!("unlink")
        .arg(dir.path().join("no-such.txt"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("unlink: cannot unlink"));
}

#[test]
fn refuses_to_remove_a_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("adir");
    fs::create_dir(&sub).unwrap();

    cargo_bin_cmd!("unlink")
        .arg(&sub)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("unlink: cannot unlink"));
    assert!(sub.is_dir(), "the directory must survive");
}

#[test]
fn double_dash_allows_dashed_file_names() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("-dash.txt");
    fs::write(&target, "data").unwrap();

    cargo_bin_cmd!("unlink")
        .args(["--", "-dash.txt"])
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(!target.exists(), "the dashed file must be gone");
}

#[test]
fn no_operands_reports_missing_operand() {
    cargo_bin_cmd!("unlink")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("unlink: missing operand"));
}

#[test]
fn two_operands_report_extra_operand() {
    cargo_bin_cmd!("unlink")
        .args(["a", "b"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("unlink: extra operand 'b'"));
}
