use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::Path;

fn write_bytes(dir: &Path, name: &str, bytes: &[u8]) -> String {
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn brief_omits_filename_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_bytes(dir.path(), "x.txt", b"hello\n");

    cargo_bin_cmd!("file")
        .arg("-b")
        .arg(&p)
        .assert()
        .success()
        .stdout("ASCII text\n");

    cargo_bin_cmd!("file")
        .arg(&p)
        .assert()
        .success()
        .stdout(format!("{}: ASCII text\n", p));
}

#[test]
fn gzip_magic_is_gzip_and_zlib_magic_is_zlib() {
    let dir = tempfile::tempdir().unwrap();
    let gz = write_bytes(
        dir.path(),
        "a.gz",
        &[0x1F, 0x8B, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03],
    );
    cargo_bin_cmd!("file")
        .arg(&gz)
        .assert()
        .success()
        .stdout(format!("{}: gzip compressed data\n", gz));

    let zz = write_bytes(
        dir.path(),
        "a.zlib",
        &[0x78, 0x9C, 0x4B, 0xCB, 0xCC, 0x49, 0x05, 0x00],
    );
    cargo_bin_cmd!("file")
        .arg(&zz)
        .assert()
        .success()
        .stdout(format!("{}: zlib compressed data\n", zz));
}

#[test]
fn keeps_going_after_unopenable_operand_and_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let good = write_bytes(dir.path(), "good.txt", b"plain text\n");

    cargo_bin_cmd!("file")
        .arg(&missing)
        .arg(&subdir)
        .arg(&good)
        .assert()
        .code(1)
        .stdout(format!("{}: ASCII text\n", good))
        .stderr(
            contains("cannot open")
                .and(contains("missing.txt"))
                .and(contains("subdir")),
        );
}

#[test]
fn plain_ascii_text_is_not_data() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_bytes(
        dir.path(),
        "notes.txt",
        b"The quick brown fox\njumps over the lazy dog\n",
    );
    cargo_bin_cmd!("file")
        .arg(&p)
        .assert()
        .success()
        .stdout(format!("{}: ASCII text\n", p));
}

#[test]
fn utf8_text_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_bytes(dir.path(), "utf8.txt", "héllo wörld\n中文测试\n".as_bytes());
    cargo_bin_cmd!("file")
        .arg(&p)
        .assert()
        .success()
        .stdout(format!("{}: Unicode text, UTF-8 text\n", p));
}
