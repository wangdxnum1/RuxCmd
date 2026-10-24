use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
#[test]
fn removes_only_requested_file_and_reports_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("remove.txt");
    let keep = dir.path().join("keep.txt");
    fs::write(&target, "remove").unwrap();
    fs::write(&keep, "keep").unwrap();
    cargo_bin_cmd!("rm").arg(&target).assert().success();
    assert!(!target.exists());
    assert_eq!(fs::read_to_string(&keep).unwrap(), "keep");
    cargo_bin_cmd!("rm").arg(&target).assert().code(1);
}

#[test]
fn force_reports_remove_failures_and_exits_nonzero() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked.txt");
    fs::write(&locked, "data").unwrap();

    // Hold the file open without FILE_SHARE_DELETE so DeleteFileW fails
    // with a sharing violation — an EPERM-class removal failure. (Read-only
    // files no longer fail: current std remove_file clears the attribute,
    // matching GNU unlink semantics.)
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    let _guard = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&locked)
        .unwrap();

    // GNU -f: failures other than ENOENT are reported and exit 1.
    cargo_bin_cmd!("rm")
        .arg("-f")
        .arg(&locked)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot remove"));
    assert!(
        locked.exists(),
        "locked file must survive the failed removal"
    );
    drop(_guard);
}

#[test]
fn force_reports_directory_without_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).unwrap();

    // Used to exit 1 silently under -f; the error must be visible.
    cargo_bin_cmd!("rm")
        .arg("-f")
        .arg(&nested)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("Is a directory"));
    assert!(nested.is_dir(), "directory must survive without -r");
}

#[test]
fn force_silently_ignores_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rm")
        .arg("-f")
        .arg(dir.path().join("no-such-file.txt"))
        .assert()
        .success()
        .code(0);
}

#[test]
fn force_does_not_suppress_preserve_root_refusal() {
    // `rm -f C:\` hits the drive-root guard; the refusal must still be
    // printed and exit 1 (and nothing is deleted).
    cargo_bin_cmd!("rm")
        .arg("-f")
        .arg("C:\\")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("refusing to remove"));
}
