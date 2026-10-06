use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
#[test]
fn copies_file_and_rejects_missing_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("source.txt");
    let dst = dir.path().join("destination.txt");
    fs::write(&src, "contents").unwrap();
    cargo_bin_cmd!("cp").arg(&src).arg(&dst).assert().success();
    assert_eq!(fs::read_to_string(&src).unwrap(), "contents");
    assert_eq!(fs::read_to_string(&dst).unwrap(), "contents");
    cargo_bin_cmd!("cp")
        .arg(dir.path().join("missing"))
        .arg(&dst)
        .assert()
        .failure();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "contents");
}
