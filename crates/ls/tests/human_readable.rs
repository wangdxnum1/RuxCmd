//! `-h` must be equivalent to `--human-readable`.
//!
//! GNU ls gives `-h` the "human-readable sizes" meaning, which collides with
//! clap's built-in help flag. These tests pin the collision fix: `-h` selects
//! human-readable sizes (K/M/G), and `--help` stays available long-only.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    // 2048 bytes renders as "2.0K" in human-readable output.
    fs::write(dir.path().join("kilobyte.bin"), vec![b'a'; 2048]).expect("write fixture file");
    // 3 bytes stays a raw byte count.
    fs::write(dir.path().join("tiny.txt"), b"abc").expect("write fixture file");
    dir
}

fn run_ls(dir: &TempDir, args: &[&str]) -> String {
    let output = cargo_bin_cmd!("ls")
        .args(args)
        .arg(dir.path())
        .output()
        .expect("run ls");
    assert!(
        output.status.success(),
        "ls {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Extract the size field (5th column: perms nlink owner group size ...) of
/// the long-format line naming `entry`.
fn size_field(listing: &str, entry: &str) -> String {
    let line = listing
        .lines()
        .find(|l| l.split_whitespace().last() == Some(entry))
        .unwrap_or_else(|| panic!("no long-format line for {entry} in: {listing}"));
    let tokens: Vec<&str> = line.split_whitespace().collect();
    assert!(tokens.len() >= 9, "long-format line truncated: {line}");
    tokens[4].to_owned()
}

#[test]
fn short_h_matches_long_human_readable() {
    let dir = fixture();
    let short = run_ls(&dir, &["-l", "-h"]);
    let long = run_ls(&dir, &["-l", "--human-readable"]);
    assert_eq!(
        short, long,
        "-h and --human-readable must produce identical output"
    );
}

#[test]
fn short_h_renders_known_sizes() {
    let dir = fixture();
    let out = run_ls(&dir, &["-l", "-h"]);
    assert_eq!(
        size_field(&out, "kilobyte.bin"),
        "2.0K",
        "2048-byte file must render human-readable"
    );
    assert_eq!(
        size_field(&out, "tiny.txt"),
        "3",
        "sub-kilobyte file must stay a raw byte count"
    );
}

#[test]
fn long_help_still_works() {
    let output = cargo_bin_cmd!("ls")
        .arg("--help")
        .output()
        .expect("run ls --help");
    assert!(
        output.status.success(),
        "--help must still exit successfully"
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("--human-readable"),
        "--help output must document --human-readable: {text}"
    );
}
