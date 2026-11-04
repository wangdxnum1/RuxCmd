use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
#[test]
fn moves_file_and_rejects_missing_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("source.txt");
    let dst = dir.path().join("destination.txt");
    fs::write(&src, "contents").unwrap();
    cargo_bin_cmd!("mv").arg(&src).arg(&dst).assert().success();
    assert!(!src.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "contents");
    cargo_bin_cmd!("mv").arg(&src).arg(&dst).assert().failure();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "contents");
}

#[test]
fn force_move_file_onto_directory_target_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("file.txt");
    fs::write(&file, "content").unwrap();

    // `dir/file.txt` is a non-empty directory: GNU mv refuses to overwrite a
    // directory with a non-directory and must never delete the tree.
    let nested = dir.path().join("dir").join("file.txt");
    fs::create_dir_all(nested.join("deep")).unwrap();
    fs::write(nested.join("deep").join("data.txt"), "keep").unwrap();

    cargo_bin_cmd!("mv")
        .arg("-f")
        .arg(&file)
        .arg(dir.path().join("dir"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot overwrite directory"));

    // Source and the entire destination tree must survive untouched.
    assert_eq!(fs::read_to_string(&file).unwrap(), "content");
    assert_eq!(
        fs::read_to_string(nested.join("deep").join("data.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn force_move_dir_onto_existing_dir_does_not_delete_tree() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("sub").join("n.txt"), "N").unwrap();

    // `dst/src` already holds data: even with -f the existing tree must not
    // be removed before the move.
    let dst = dir.path().join("dst");
    fs::create_dir_all(dst.join("src")).unwrap();
    fs::write(dst.join("src").join("old.txt"), "OLD").unwrap();

    cargo_bin_cmd!("mv")
        .arg("-f")
        .arg(&src)
        .arg(&dst)
        .assert()
        .failure()
        .code(1);

    assert_eq!(
        fs::read_to_string(dst.join("src").join("old.txt")).unwrap(),
        "OLD"
    );
    assert!(
        src.join("sub").join("n.txt").exists(),
        "source must survive"
    );
}

#[test]
fn move_directory_onto_existing_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("srcdir");
    fs::create_dir_all(src_dir.join("sub")).unwrap();
    fs::write(src_dir.join("sub").join("n.txt"), "N").unwrap();

    let dst_file = dir.path().join("dst.txt");
    fs::write(&dst_file, "D").unwrap();

    cargo_bin_cmd!("mv")
        .arg("-f")
        .arg(&src_dir)
        .arg(&dst_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot overwrite non-directory"));

    assert!(src_dir.is_dir(), "source directory must survive");
    assert_eq!(fs::read_to_string(&dst_file).unwrap(), "D");
}

#[test]
fn moves_file_into_directory_by_bare_name() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.txt");
    fs::write(&file, "A").unwrap();
    let out = dir.path().join("out");
    fs::create_dir(&out).unwrap();
    cargo_bin_cmd!("mv").arg(&file).arg(&out).assert().success();
    assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "A");
    assert!(!file.exists());
}

fn set_readonly(path: &std::path::Path) {
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn force_replaces_readonly_destination() {
    // GNU mv replaces the destination regardless of its own write bit
    // (unlink depends on the directory), so -f must clear the Windows
    // read-only attribute instead of failing with "Access is denied".
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    fs::write(&src, "NEW").unwrap();
    fs::write(&dst, "OLD").unwrap();
    set_readonly(&dst);

    cargo_bin_cmd!("mv")
        .arg("-f")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    assert!(!src.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "NEW");
    assert!(
        !fs::metadata(&dst).unwrap().permissions().readonly(),
        "the replacement must not inherit the destination's read-only attribute"
    );
}

#[test]
fn interactive_yes_replaces_readonly_destination() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    fs::write(&src, "NEW").unwrap();
    fs::write(&dst, "OLD").unwrap();
    set_readonly(&dst);

    cargo_bin_cmd!("mv")
        .arg("-i")
        .arg(&src)
        .arg(&dst)
        .write_stdin("y\n")
        .assert()
        .success();

    assert!(!src.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "NEW");
}

#[test]
fn no_clobber_keeps_readonly_destination() {
    // `-n` never reaches the delete step, so the read-only bit must survive
    // untouched along with the source. (This port reports the skip as a
    // "File exists" error instead of GNU's silent exit 0; that difference is
    // pre-existing and unrelated to the read-only attribute.)
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    fs::write(&src, "NEW").unwrap();
    fs::write(&dst, "OLD").unwrap();
    set_readonly(&dst);

    cargo_bin_cmd!("mv")
        .arg("-n")
        .arg(&src)
        .arg(&dst)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("File exists"));

    assert_eq!(fs::read_to_string(&dst).unwrap(), "OLD");
    assert!(fs::metadata(&dst).unwrap().permissions().readonly());
    assert!(src.exists(), "the source must be kept");
}

#[test]
fn plain_move_refuses_readonly_destination() {
    // Without -f this port refuses any existing destination; a read-only one
    // must give the regular "File exists" report, not an OS error, and must
    // keep its content and attributes.
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    fs::write(&src, "NEW").unwrap();
    fs::write(&dst, "OLD").unwrap();
    set_readonly(&dst);

    cargo_bin_cmd!("mv")
        .arg(&src)
        .arg(&dst)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("File exists"));

    assert_eq!(fs::read_to_string(&dst).unwrap(), "OLD");
    assert!(fs::metadata(&dst).unwrap().permissions().readonly());
    assert!(src.exists());
}
