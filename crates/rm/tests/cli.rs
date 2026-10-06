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
