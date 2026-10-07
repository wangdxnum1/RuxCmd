//! Regression tests for P0 #34: extraction must not follow unsafe entry
//! paths (`..` traversal, absolute paths) outside of the extraction
//! directory ("zip slip"). Unsafe entries must be rejected with a warning
//! and a non-zero exit code, while benign entries still extract.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Builds an archive with one benign entry and two malicious entries: a
/// `..` traversal and a POSIX-style absolute path. The zip crate's writer
/// stores entry names verbatim, so this produces exactly the archives a
/// real attacker would craft.
fn build_malicious_archive(path: &Path) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    zip.start_file("ok.txt", options).unwrap();
    zip.write_all(b"benign").unwrap();

    zip.start_file("../../evil.txt", options).unwrap();
    zip.write_all(b"traversal").unwrap();

    zip.start_file("/abs.txt", options).unwrap();
    zip.write_all(b"absolute").unwrap();

    zip.finish().unwrap();
}

#[test]
fn extract_rejects_traversal_and_absolute_entries() {
    let base = tempfile::tempdir().unwrap();
    // Extraction runs two levels deep, so `../../evil.txt` would land in
    // `base` (outside the extraction directory) instead of outside the
    // tempdir root itself.
    let work = base.path().join("extract").join("target");
    fs::create_dir_all(&work).unwrap();

    let archive = base.path().join("malicious.zip");
    build_malicious_archive(&archive);

    // Where `../../evil.txt` would escape to when extracting into `work`.
    let traversal_escape = base.path().join("evil.txt");
    // A zip entry with a leading `/` is joined by replacing the whole
    // destination path, so it would land on the current drive root.
    // Only inspect that root path if nothing pre-exists there; a user
    // file must never be touched or failed on.
    let had_preexisting_root_file = Path::new(r"C:\abs.txt").exists();

    let assert = cargo_bin_cmd!("unzip")
        .arg("-d")
        .arg(&work)
        .arg(&archive)
        .current_dir(&base)
        .assert();

    // Record, then remove, any drive-root file an unfixed binary created
    // during a failing regression run, so no run leaves root litter and
    // the assertion below still proves the escape (or its absence).
    let escaped_to_root = !had_preexisting_root_file && Path::new(r"C:\abs.txt").exists();
    if !had_preexisting_root_file {
        let _ = fs::remove_file(r"C:\abs.txt");
    }

    assert
        .code(2)
        .stderr(predicates::str::contains("../../evil.txt"))
        .stderr(predicates::str::contains("/abs.txt"));

    assert!(
        !traversal_escape.exists(),
        "traversal entry escaped the extraction directory: {}",
        traversal_escape.display()
    );
    assert!(
        !escaped_to_root,
        "absolute entry escaped to the drive root"
    );
    // The benign entry is still extracted, so rejection skips the entry
    // instead of aborting the whole archive.
    assert_eq!(fs::read(work.join("ok.txt")).unwrap(), b"benign");
}

#[test]
fn extract_still_works_for_benign_archive() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("out");

    let archive = base.path().join("benign.zip");
    {
        let file = File::create(&archive).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("file.txt", zip::write::FileOptions::default())
            .unwrap();
        zip.write_all(b"payload").unwrap();
        zip.finish().unwrap();
    }

    cargo_bin_cmd!("unzip")
        .arg("-d")
        .arg(&work)
        .arg(&archive)
        .current_dir(&base)
        .assert()
        .success();

    assert_eq!(fs::read(work.join("file.txt")).unwrap(), b"payload");
}
