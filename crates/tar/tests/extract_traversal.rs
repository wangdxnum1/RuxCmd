//! Regression tests for P0 #32: extraction must not follow unsafe entry
//! paths (`..` traversal, absolute paths, drive prefixes) outside of the
//! extraction directory.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Writes one classic ustar header plus payload with an arbitrary,
/// unvalidated entry name. The tar crate's writer (`set_path`) refuses to
/// encode unsafe names, but a real malicious archive is crafted as raw
/// header bytes, so the fixture does exactly that.
fn append_raw_ustar_entry(file: &mut File, name: &str, data: &[u8]) {
    let mut header = [0u8; 512];
    header[..name.len()].copy_from_slice(name.as_bytes());
    header[100..108].copy_from_slice(b"0000644\0"); // mode
    header[108..116].copy_from_slice(b"0000000\0"); // uid
    header[116..124].copy_from_slice(b"0000000\0"); // gid
    header[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes()); // size
    header[136..148].copy_from_slice(b"00000000000\0"); // mtime
    header[156] = b'0'; // typeflag: regular file
    header[257..263].copy_from_slice(b"ustar\0"); // magic
    header[263..265].copy_from_slice(b"00"); // version
    header[148..156].copy_from_slice(b"        "); // checksum field: spaces
    let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
    header[148..156].copy_from_slice(format!("{:06o}\0 ", sum).as_bytes());

    file.write_all(&header).unwrap();
    file.write_all(data).unwrap();
    let pad = (512 - data.len() % 512) % 512;
    file.write_all(&vec![0u8; pad]).unwrap();
}

/// Builds an archive with one benign entry (via the tar crate writer, so
/// normal archives stay unaffected) and two malicious entries as raw bytes:
/// a `..` traversal and a Windows drive-prefixed absolute path.
fn build_malicious_archive(path: &Path) {
    // The tar crate's writer cannot encode unsafe names, and `into_inner`
    // unconditionally appends the end-of-archive marker, so the benign
    // entry is produced with the Builder (marker stripped afterwards) and
    // the malicious entries are appended before the final marker.
    let mut benign: Vec<u8> = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut benign);
        let mut header = tar::Header::new_gnu();
        header.set_size(b"benign".len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        builder
            .append_data(&mut header, "ok.txt", b"benign".as_slice())
            .unwrap();
        builder.into_inner().unwrap();
    }
    assert!(benign.len() >= 1024);
    benign.truncate(benign.len() - 1024); // strip Builder's end-of-archive marker

    let mut file = File::create(path).unwrap();
    file.write_all(&benign).unwrap();
    append_raw_ustar_entry(&mut file, "../../evil.txt", b"traversal");
    append_raw_ustar_entry(&mut file, "C:\\evil.txt", b"absolute");
    file.write_all(&[0u8; 1024]).unwrap();
}

#[test]
fn extract_rejects_traversal_and_absolute_entries() {
    let base = tempfile::tempdir().unwrap();
    // Extraction runs two levels deep, so `../../evil.txt` would land in
    // `base` (outside the extraction directory) instead of outside the
    // tempdir root itself.
    let work = base.path().join("extract").join("target");
    fs::create_dir_all(&work).unwrap();

    let archive = base.path().join("malicious.tar");
    build_malicious_archive(&archive);

    // Where `../../evil.txt` would escape to when running inside `work`.
    let traversal_escape = base.path().join("evil.txt");
    // Only assert the drive-root entry was refused if nothing pre-exists at
    // that exact path; a user file must never be touched or failed on.
    let had_preexisting_root_file = Path::new(r"C:\evil.txt").exists();

    cargo_bin_cmd!("tar")
        .arg("-x")
        .arg("-f")
        .arg(&archive)
        .current_dir(&work)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("../../evil.txt"))
        .stderr(predicates::str::contains(r"C:\evil.txt"));

    assert!(
        !traversal_escape.exists(),
        "traversal entry escaped the extraction directory: {}",
        traversal_escape.display()
    );
    if !had_preexisting_root_file {
        assert!(
            !Path::new(r"C:\evil.txt").exists(),
            "absolute entry escaped to the drive root"
        );
    }
    // The benign entry is still extracted, so rejection skips the entry
    // instead of aborting the whole archive.
    assert_eq!(fs::read(work.join("ok.txt")).unwrap(), b"benign");
}

#[test]
fn extract_still_works_for_benign_archive() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("out");
    fs::create_dir_all(&work).unwrap();

    let archive = base.path().join("benign.tar");
    let file = File::create(&archive).unwrap();
    let mut builder = tar::Builder::new(file);
    let mut header = tar::Header::new_gnu();
    header.set_size(b"payload".len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_cksum();
    builder
        .append_data(&mut header, "file.txt", b"payload".as_slice())
        .unwrap();
    builder.into_inner().unwrap();

    cargo_bin_cmd!("tar")
        .arg("-x")
        .arg("-f")
        .arg(&archive)
        .current_dir(&work)
        .assert()
        .success();

    assert_eq!(fs::read(work.join("file.txt")).unwrap(), b"payload");
}
