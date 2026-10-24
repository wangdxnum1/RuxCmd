//! Regression tests for P0 #14: touch on an existing file must actually
//! update timestamps (SetFileTime), directories and read-only files must be
//! touchable, and `-c` must never create files.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::fs::FileTimes;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Open `path` for writing and force its timestamps to the given value so we
/// do not depend on sleep-based timing.
fn force_times(path: &Path, atime: SystemTime, mtime: SystemTime) {
    let f = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for pre-setting timestamps");
    f.set_times(FileTimes::new().set_accessed(atime).set_modified(mtime))
        .expect("pre-set timestamps");
}

fn mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .expect("metadata after touch")
        .modified()
        .expect("mtime after touch")
}

fn atime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .expect("metadata after touch")
        .accessed()
        .expect("atime after touch")
}

// --- Core P0: touching an existing file must change its timestamps ---

#[test]
fn touch_existing_file_updates_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);
    let before = mtime(&file);

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    let after = mtime(&file);
    assert!(
        after > before,
        "mtime must advance after touching an existing file (before={before:?}, after={after:?})"
    );
    // The file content must be preserved.
    assert_eq!(fs::read(&file).unwrap(), b"payload");
}

#[test]
fn touch_existing_file_updates_atime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing-atime.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);
    let before = atime(&file);

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    let after = atime(&file);
    assert!(
        after > before,
        "atime must advance after touching an existing file (before={before:?}, after={after:?})"
    );
}

#[test]
fn touch_m_updates_mtime_but_leaves_atime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("m-only.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-m")
        .arg(&file)
        .assert()
        .success();

    assert!(mtime(&file) > old, "-m must update mtime");
    assert!(
        atime(&file) <= old + Duration::from_secs(2),
        "-m must not update atime (got {:?}, pre-set {:?})",
        atime(&file),
        old
    );
}

#[test]
fn touch_a_updates_atime_but_leaves_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a-only.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-a")
        .arg(&file)
        .assert()
        .success();

    assert!(atime(&file) > old, "-a must update atime");
    assert!(
        mtime(&file) <= old + Duration::from_secs(2),
        "-a must not update mtime (got {:?}, pre-set {:?})",
        mtime(&file),
        old
    );
}

// --- -c / --no-create ---

#[test]
fn no_create_does_not_create_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");

    cargo_bin_cmd!("touch")
        .arg("-c")
        .arg(&missing)
        .assert()
        .success();
    assert!(!missing.exists(), "-c must not create the file");

    cargo_bin_cmd!("touch")
        .arg("--no-create")
        .arg(&missing)
        .assert()
        .success();
    assert!(!missing.exists(), "--no-create must not create the file");
}

#[test]
fn no_create_on_existing_file_still_updates_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-c")
        .arg(&file)
        .assert()
        .success();

    assert!(
        mtime(&file) > old,
        "-c on an existing file must still update mtime"
    );
}

// --- Directories and read-only files ---

#[test]
fn touch_directory_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    fs::create_dir(&sub).unwrap();

    cargo_bin_cmd!("touch").arg(&sub).assert().success();
}

#[test]
fn touch_readonly_file_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readonly.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    // Make the file read-only, like `attrib +r`.
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file, perms).unwrap();

    cargo_bin_cmd!("touch").arg(&file).assert().success();
    assert!(mtime(&file) > old, "mtime must advance on a read-only file");

    // Restore write access so the temp dir can be cleaned up; clearing the
    // read-only bit is the whole point here.
    let mut perms = fs::metadata(&file).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(&file, perms).unwrap();
}

// --- Creation of missing files ---

#[test]
fn touch_missing_file_creates_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("created.txt");
    assert!(!file.exists());

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    assert!(file.exists(), "touch must create the file");
    let meta = fs::metadata(&file).unwrap();
    assert!(meta.is_file());
    assert_eq!(meta.len(), 0, "created file must be empty");
}

#[test]
fn touch_multiple_files_updates_all() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    fs::write(&a, b"").unwrap();
    fs::write(&b, b"").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&a, old, old);
    force_times(&b, old, old);

    cargo_bin_cmd!("touch").arg(&a).arg(&b).assert().success();

    assert!(mtime(&a) > old, "first file must be touched");
    assert!(mtime(&b) > old, "second file must be touched");
}

#[test]
fn missing_operand_fails_with_error() {
    cargo_bin_cmd!("touch")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("missing file operand"));
}

#[test]
fn unwritable_path_fails_with_error() {
    cargo_bin_cmd!("touch")
        .arg("Z:\\no\\such\\directory\\file.txt")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("touch:"));
}
