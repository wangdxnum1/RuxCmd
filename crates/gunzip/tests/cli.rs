//! Round 12 I1 tests: `-c/--stdout`, `-l/--list`, and multi-member
//! (concatenated gzip) streams.
//!
//! Uses std `Command` + `CARGO_BIN_EXE_*` (no dev-dependencies) so the
//! crate's dependency set stays untouched.

use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn gunzip_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gunzip"))
}

/// Creates an isolated directory `target/`-free unique temp path; tests
/// clean up after themselves.
fn temp_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = base.join(format!(
        "gunzip-test-{}-{}-{}",
        tag,
        unique,
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn gzip_bytes(data: &[u8]) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

fn write_gz(path: &Path, members: &[&[u8]]) {
    let mut raw: Vec<u8> = Vec::new();
    for member in members {
        raw.extend_from_slice(&gzip_bytes(member));
    }
    fs::write(path, &raw).unwrap();
}

/// `-c` decompresses to stdout and never deletes the source file.
#[test]
fn stdout_flag_writes_content_and_keeps_source() {
    let dir = temp_dir("stdout");
    let gz = dir.join("data.gz");
    write_gz(&gz, &[b"hello stdout\n"]);

    let out = gunzip_cmd()
        .args(["-c"])
        .arg(&gz)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello stdout\n");

    assert!(gz.exists(), "-c must keep the source archive");
    assert!(
        !dir.join("data").exists(),
        "-c must not write an output file"
    );

    fs::remove_dir_all(&dir).unwrap();
}

/// Concatenated gzip members decompress fully (single-member decoders
/// truncate after the first member).
#[test]
fn multi_member_stream_decompresses_completely() {
    let dir = temp_dir("multi");
    let gz = dir.join("multi.gz");
    write_gz(&gz, &[b"first-part;", b"second-part"]);

    let out = gunzip_cmd().arg(&gz).current_dir(&dir).output().unwrap();
    assert!(out.status.success(), "stderr: {:?}", out.stderr);

    assert_eq!(
        fs::read(dir.join("multi")).unwrap(),
        b"first-part;second-part"
    );
    assert!(!gz.exists(), "default mode removes the source");

    fs::remove_dir_all(&dir).unwrap();
}

/// `-c` handles multi-member streams too.
#[test]
fn multi_member_stream_to_stdout() {
    let dir = temp_dir("multi-stdout");
    let gz = dir.join("multi.gz");
    write_gz(&gz, &[b"AAA", b"BBB"]);

    let out = gunzip_cmd()
        .args(["-c"])
        .arg(&gz)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    assert_eq!(out.stdout, b"AAABBB");

    fs::remove_dir_all(&dir).unwrap();
}

/// `-l` reports compressed and uncompressed sizes.
#[test]
fn list_prints_compressed_and_uncompressed_sizes() {
    let dir = temp_dir("list");
    let gz = dir.join("data.gz");
    let payload = b"0123456789".repeat(100); // 1000 bytes
    write_gz(&gz, &[payload.as_slice()]);

    let compressed_len = fs::metadata(&gz).unwrap().len();

    let out = gunzip_cmd()
        .args(["-l"])
        .arg(&gz)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains(&compressed_len.to_string()),
        "stdout was: {}",
        stdout
    );
    assert!(stdout.contains("1000"), "stdout was: {}", stdout);

    fs::remove_dir_all(&dir).unwrap();
}
