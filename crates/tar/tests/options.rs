//! Round 12 I1 tests: `-C/--directory`, `-O/--to-stdout`, `--exclude`,
//! and member-name selection for extract/list (FILES positionals).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Builds a plain (uncompressed) tar with the given name/content entries.
fn make_archive(path: &Path, files: &[(&str, &str)]) {
    let file = File::create(path).unwrap();
    let mut builder = tar::Builder::new(file);
    for (name, content) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        builder
            .append_data(&mut header, name, content.as_bytes())
            .unwrap();
    }
    builder.into_inner().unwrap();
}

/// `-C DIR` on create must read operands relative to DIR while storing
/// names relative to DIR (chdir semantics); on extract it must unpack
/// into DIR.
#[test]
fn directory_flag_create_and_extract() {
    let base = tempfile::tempdir().unwrap();
    let src = base.path().join("src");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("sub/a.txt"), b"alpha").unwrap();
    fs::write(src.join("b.txt"), b"beta").unwrap();

    let archive = base.path().join("arc.tar");
    cargo_bin_cmd!("tar")
        .args(["-c", "-f"])
        .arg(&archive)
        .args(["-C", "src", "sub", "b.txt"])
        .current_dir(base.path())
        .assert()
        .success();

    // Names in the archive are relative to the -C directory.
    cargo_bin_cmd!("tar")
        .args(["-t", "-f"])
        .arg(&archive)
        .current_dir(base.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("sub/a.txt"))
        .stdout(predicates::str::contains("b.txt"));

    // Extraction with -C unpacks into the directory.
    cargo_bin_cmd!("tar")
        .args(["-x", "-f"])
        .arg(&archive)
        .args(["-C", "out"])
        .current_dir(base.path())
        .assert()
        .success();

    assert_eq!(fs::read(base.path().join("out/sub/a.txt")).unwrap(), b"alpha");
    assert_eq!(fs::read(base.path().join("out/b.txt")).unwrap(), b"beta");
}

/// `-O` writes entry contents to stdout instead of creating files.
#[test]
fn to_stdout_prints_contents_without_files() {
    let base = tempfile::tempdir().unwrap();
    let work = base.path().join("work");
    fs::create_dir_all(&work).unwrap();
    let archive = base.path().join("arc.tar");
    make_archive(&archive, &[("one.txt", "first\n"), ("two.txt", "second\n")]);

    cargo_bin_cmd!("tar")
        .args(["-x", "-O", "-f"])
        .arg(&archive)
        .arg("two.txt")
        .current_dir(&work)
        .assert()
        .success()
        .stdout("second\n");

    assert!(
        !work.join("two.txt").exists(),
        "-O must not create files on disk"
    );
    assert!(!work.join("one.txt").exists(), "member selection must apply");
}

/// `--exclude PATTERN` (glob with `*`/`?`) skips matching entries during
/// creation, at any depth for slash-free patterns.
#[test]
fn exclude_pattern_skips_entries_on_create() {
    let base = tempfile::tempdir().unwrap();
    let src = base.path().join("src");
    fs::create_dir_all(src.join("nested/deep")).unwrap();
    fs::write(src.join("keep.txt"), b"k").unwrap();
    fs::write(src.join("skip.log"), b"s").unwrap();
    fs::write(src.join("nested/inner.log"), b"i").unwrap();
    fs::write(src.join("nested/deep/lowest.log"), b"l").unwrap();

    let archive = base.path().join("arc.tar");
    cargo_bin_cmd!("tar")
        .args(["-c", "-f"])
        .arg(&archive)
        .args(["--exclude", "*.log", "src"])
        .current_dir(base.path())
        .assert()
        .success();

    let listing = {
        let output = cargo_bin_cmd!("tar")
            .args(["-t", "-f"])
            .arg(&archive)
            .current_dir(base.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(output).unwrap()
    };

    assert!(listing.contains("keep.txt"), "listing was: {}", listing);
    assert!(!listing.contains(".log"), "listing was: {}", listing);
}

/// `--exclude` also applies on extraction: matching entries are skipped.
#[test]
fn exclude_pattern_skips_entries_on_extract() {
    let base = tempfile::tempdir().unwrap();
    let archive = base.path().join("arc.tar");
    make_archive(&archive, &[("a.txt", "a"), ("b.log", "b")]);

    cargo_bin_cmd!("tar")
        .args(["-x", "-f"])
        .arg(&archive)
        .args(["--exclude", "*.log"])
        .current_dir(base.path())
        .assert()
        .success();

    assert!(base.path().join("a.txt").exists());
    assert!(!base.path().join("b.log").exists());
}

/// FILES positionals on extract/list select members by exact name or
/// directory prefix.
#[test]
fn member_names_select_entries() {
    let base = tempfile::tempdir().unwrap();
    let archive = base.path().join("arc.tar");
    make_archive(
        &archive,
        &[("one.txt", "1"), ("dir/two.txt", "2"), ("dir/three.txt", "3")],
    );

    // List only a directory member: prefix match includes its children.
    let listing = cargo_bin_cmd!("tar")
        .args(["-t", "-f"])
        .arg(&archive)
        .arg("dir")
        .current_dir(base.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listing = String::from_utf8(listing).unwrap();
    assert!(listing.contains("dir/two.txt"), "listing was: {}", listing);
    assert!(listing.contains("dir/three.txt"), "listing was: {}", listing);
    assert!(!listing.contains("one.txt"), "listing was: {}", listing);

    // Extract only one member by exact name.
    cargo_bin_cmd!("tar")
        .args(["-x", "-f"])
        .arg(&archive)
        .arg("one.txt")
        .current_dir(base.path())
        .assert()
        .success();
    assert!(base.path().join("one.txt").exists());
    assert!(!base.path().join("dir").exists(), "only selected members extract");
}

/// Extraction of a selected member writes correct content (guards against
/// member filtering corrupting offsets in the streaming reader).
#[test]
fn selected_member_content_is_correct() {
    let base = tempfile::tempdir().unwrap();
    let archive = base.path().join("arc.tar");
    make_archive(
        &archive,
        &[("first.txt", "first-payload"), ("second.txt", "second-payload")],
    );

    cargo_bin_cmd!("tar")
        .args(["-x", "-f"])
        .arg(&archive)
        .arg("second.txt")
        .current_dir(base.path())
        .assert()
        .success();

    assert_eq!(
        fs::read(base.path().join("second.txt")).unwrap(),
        b"second-payload"
    );
}

/// Helper used by doc above; keeps `Write` import meaningful on all cfgs.
#[allow(dead_code)]
fn touch(path: &Path) {
    let mut f = File::create(path).unwrap();
    f.write_all(b"").unwrap();
}
