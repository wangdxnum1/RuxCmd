//! Regression tests for P0 #25 (Task 40):
//! 1. Alphabetic suffixes must follow GNU order aa..az, ba.. (the old
//!    generator produced x0a..x0z because it left-padded with '0').
//! 2. Line mode must be byte-exact: CRLF preserved, no newline added to a
//!    final unterminated line, non-UTF-8 input split safely.
//! 3. Suffix exhaustion (more pieces than 26^N / 10^N) must error with
//!    `output file suffixes exhausted` instead of silently widening.
//! 4. `-l 0` must be rejected.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

fn split_cmd() -> assert_cmd::Command {
    cargo_bin_cmd!("split")
}

/// The first 27 suffixes with default width 2 must be aa..az, ba — not
/// x0a..x0z. Every produced file must carry exactly its own line.
#[test]
fn alphabetic_suffixes_follow_gnu_order() {
    let dir = tempfile::tempdir().unwrap();
    let input: String = (0..27).map(|i| format!("line{}\n", i)).collect();
    std::fs::write(dir.path().join("in.txt"), input).unwrap();

    split_cmd()
        .args(["-l", "1", "-a", "2", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    let mut want: Vec<String> = Vec::new();
    'outer: for hi in b'a'..=b'z' {
        for lo in b'a'..=b'z' {
            want.push(format!("p{}{}", hi as char, lo as char));
            if want.len() == 27 {
                break 'outer;
            }
        }
    }

    for (i, name) in want.iter().enumerate() {
        let path = dir.path().join(name);
        let content = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("missing or unreadable {}: {}", name, e));
        assert_eq!(
            content,
            format!("line{}\n", i).into_bytes(),
            "wrong content in {}",
            name
        );
    }

    // Exactly 27 output files, and the mis-padded shape must not exist.
    let count = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with('p'))
        .count();
    assert_eq!(count, 27, "expected exactly 27 chunk files");
    assert!(!dir.path().join("p0a").exists(), "zero-padded suffix leaked");
}

/// Numeric suffixes (-d) are zero-padded decimal: 00..09, 10.
#[test]
fn numeric_suffixes_are_zero_padded_decimal() {
    let dir = tempfile::tempdir().unwrap();
    let input: String = (0..11).map(|i| format!("n{}\n", i)).collect();
    std::fs::write(dir.path().join("in.txt"), input).unwrap();

    split_cmd()
        .args(["-l", "1", "-d", "-a", "2", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    let mut want: Vec<String> = Vec::new();
    for i in 0..11u32 {
        want.push(format!("p{:02}", i));
    }
    for name in &want {
        assert!(dir.path().join(name).exists(), "missing {}", name);
    }
    let count = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with('p'))
        .count();
    assert_eq!(count, 11);
}

/// Line mode must copy bytes verbatim: `\r` inside lines kept, final line
/// without a trailing newline not terminated, invalid UTF-8 harmless, and
/// total output bytes equal to input bytes.
#[test]
fn line_mode_is_byte_exact_with_crlf_and_non_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let input: &[u8] = b"alpha\r\nbeta\r\n\xFF\xFE\r\ntail-no-newline";
    std::fs::write(dir.path().join("in.txt"), input).unwrap();

    split_cmd()
        .args(["-l", "2", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    let first = std::fs::read(dir.path().join("paa")).unwrap();
    let second = std::fs::read(dir.path().join("pab")).unwrap();
    assert_eq!(first, b"alpha\r\nbeta\r\n");
    assert_eq!(second, b"\xFF\xFE\r\ntail-no-newline");
    assert_eq!(first.len() + second.len(), input.len(), "bytes not conserved");
}

/// A final unterminated line must not gain a newline in single-line mode.
#[test]
fn last_line_without_newline_is_not_terminated() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"one\ntwo").unwrap();

    split_cmd()
        .args(["-l", "1", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(std::fs::read(dir.path().join("paa")).unwrap(), b"one\n");
    assert_eq!(std::fs::read(dir.path().join("pab")).unwrap(), b"two");
}

/// More chunks than 26^width suffixes: error, not silent widening.
#[test]
fn alphabetic_suffix_exhaustion_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input: String = "x\n".repeat(27);
    std::fs::write(dir.path().join("in.txt"), input).unwrap();

    split_cmd()
        .args(["-l", "1", "-a", "1", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("output file suffixes exhausted"));
}

/// Numeric suffixes run out at 10^width.
#[test]
fn numeric_suffix_exhaustion_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input: String = "x\n".repeat(11);
    std::fs::write(dir.path().join("in.txt"), input).unwrap();

    split_cmd()
        .args(["-l", "1", "-d", "-a", "1", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("output file suffixes exhausted"));
}

/// `-l 0` is rejected (GNU: invalid number of lines).
#[test]
fn zero_lines_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    split_cmd()
        .args(["-l", "0", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("invalid number of lines"));
}

/// Byte mode shares the suffix generator: -b 1 on 3 bytes gives aa..ac.
#[test]
fn byte_mode_uses_gnu_suffix_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"abc").unwrap();

    split_cmd()
        .args(["-b", "1", "-o", "p", "in.txt"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(std::fs::read(dir.path().join("paa")).unwrap(), b"a");
    assert_eq!(std::fs::read(dir.path().join("pab")).unwrap(), b"b");
    assert_eq!(std::fs::read(dir.path().join("pac")).unwrap(), b"c");
}
