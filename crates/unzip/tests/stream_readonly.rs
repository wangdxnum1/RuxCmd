//! Round 12 I1 tests: streamed extraction of large entries (no full
//! in-memory buffering) and overwriting read-only targets with -o.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Deterministic pseudo-random byte stream (xorshift64*), so the expected
/// payload can be regenerated without storing or embedding 10 MB.
fn pseudo_random_stream(len: usize) -> Vec<u8> {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let x = state.wrapping_mul(0x2545F4914F6CDD1D);
        out.extend_from_slice(&x.to_le_bytes());
    }
    out.truncate(len);
    out
}

fn build_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();
    for (name, data) in entries.iter() {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

/// A 10 MB entry must extract with byte-identical content. This exercises
/// the streaming copy path (the old read_to_end buffered everything).
#[test]
fn large_entry_extracts_correctly() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("out");
    let archive = base.path().join("big.zip");

    const SIZE: usize = 10 * 1024 * 1024;
    let payload = pseudo_random_stream(SIZE);
    build_zip(&archive, &[("big.bin", &payload)]);

    cargo_bin_cmd!("unzip")
        .args(["-d"])
        .arg(&work)
        .arg(&archive)
        .current_dir(base.path())
        .assert()
        .success();

    let extracted = fs::read(work.join("big.bin")).unwrap();
    assert_eq!(extracted.len(), SIZE);
    let expected = pseudo_random_stream(SIZE);
    assert_eq!(extracted, expected, "extracted content must match payload");
}

/// `-o` must overwrite an existing read-only target by clearing the
/// read-only attribute first (cp-style), instead of failing outright.
#[test]
fn overwrite_flag_replaces_readonly_target() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("out");
    let first = base.path().join("first.zip");
    let second = base.path().join("second.zip");
    build_zip(&first, &[("f.txt", b"one".as_slice())]);
    build_zip(&second, &[("f.txt", b"two".as_slice())]);

    cargo_bin_cmd!("unzip")
        .args(["-d"])
        .arg(&work)
        .arg(&first)
        .current_dir(base.path())
        .assert()
        .success();

    let target = work.join("f.txt");
    let mut perms = fs::metadata(&target).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&target, perms).unwrap();
    assert!(fs::metadata(&target).unwrap().permissions().readonly());

    cargo_bin_cmd!("unzip")
        .args(["-d"])
        .arg(&work)
        .arg("-o")
        .arg(&second)
        .current_dir(base.path())
        .assert()
        .success();

    assert_eq!(fs::read(&target).unwrap(), b"two");
}

/// Without -o a read-only (existing) target is still skipped, content
/// unchanged, and the run reports success.
#[test]
fn readonly_target_is_skipped_without_overwrite() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("out");
    let first = base.path().join("first.zip");
    let second = base.path().join("second.zip");
    build_zip(&first, &[("f.txt", b"one".as_slice())]);
    build_zip(&second, &[("f.txt", b"two".as_slice())]);

    cargo_bin_cmd!("unzip")
        .args(["-d"])
        .arg(&work)
        .arg(&first)
        .current_dir(base.path())
        .assert()
        .success();

    let target = work.join("f.txt");
    let mut perms = fs::metadata(&target).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&target, perms).unwrap();

    cargo_bin_cmd!("unzip")
        .args(["-d"])
        .arg(&work)
        .arg(&second)
        .current_dir(base.path())
        .assert()
        .success();

    assert_eq!(
        fs::read(&target).unwrap(),
        b"one",
        "target must be untouched"
    );
    assert!(
        fs::metadata(&target).unwrap().permissions().readonly(),
        "skipped target keeps its read-only attribute"
    );
}

#[allow(dead_code)]
fn unused_path_helper(_: &Path) {}
