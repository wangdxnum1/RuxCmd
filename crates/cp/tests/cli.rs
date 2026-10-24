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

#[test]
fn force_overwrites_readonly_destination() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    fs::write(&a, "NEW").unwrap();
    let b = dir.path().join("b.txt");
    fs::write(&b, "OLD").unwrap();
    let mut perm = fs::metadata(&b).unwrap().permissions();
    perm.set_readonly(true);
    fs::set_permissions(&b, perm).unwrap();

    // Default (no -f) still refuses to overwrite.
    cargo_bin_cmd!("cp").arg(&a).arg(&b).assert().failure().code(1);
    assert_eq!(fs::read_to_string(&b).unwrap(), "OLD");

    // -f clears FILE_ATTRIBUTE_READONLY and overwrites.
    cargo_bin_cmd!("cp").arg("-f").arg(&a).arg(&b).assert().success();
    assert_eq!(fs::read_to_string(&b).unwrap(), "NEW");
}
