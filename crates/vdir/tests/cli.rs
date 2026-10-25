//! vdir must behave like `ls -l`: a long-format listing by default, with
//! extra arguments forwarded to the underlying ls engine.
//!
//! The assertions mirror the long-format column checks used by the ls
//! crate's own tests (perms nlink owner group size month day time name).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("alpha.txt"), b"hello").expect("write fixture file");
    // 2048 bytes renders as "2048" by default and "2.0K" with -h.
    fs::write(dir.path().join("kilobyte.bin"), vec![b'a'; 2048]).expect("write fixture file");
    fs::create_dir(dir.path().join("beta")).expect("create fixture dir");
    dir
}

fn run_vdir(dir: &TempDir, args: &[&str]) -> String {
    let output = cargo_bin_cmd!("vdir")
        .args(args)
        .arg(dir.path())
        .output()
        .expect("run vdir");
    assert!(
        output.status.success(),
        "vdir {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn line_for<'a>(listing: &'a str, entry: &str) -> Vec<&'a str> {
    let line = listing
        .lines()
        .find(|l| l.split_whitespace().last() == Some(entry))
        .unwrap_or_else(|| panic!("no long-format line for {entry} in: {listing}"));
    let tokens: Vec<&str> = line.split_whitespace().collect();
    // perms nlink owner group size month day time name
    assert!(tokens.len() >= 9, "long-format line truncated: {line:?}");
    tokens
}

#[test]
fn vdir_lists_long_format() {
    let dir = fixture();
    let out = run_vdir(&dir, &[]);
    // One line per entry, no "total" header (single target, like ls -l).
    assert_eq!(
        out.lines().count(),
        3,
        "one line per entry expected: {out:?}"
    );

    let alpha = line_for(&out, "alpha.txt");
    assert_eq!(alpha[0].len(), 10, "permissions field malformed: {alpha:?}");
    assert_eq!(
        alpha[0].chars().next(),
        Some('-'),
        "file type char: {alpha:?}"
    );
    assert!(
        alpha[1].bytes().all(|b| b.is_ascii_digit()),
        "nlink must be numeric: {alpha:?}"
    );
    assert_eq!(alpha[4], "5", "size column must be raw bytes: {alpha:?}");

    let beta = line_for(&out, "beta");
    assert_eq!(
        beta[0].chars().next(),
        Some('d'),
        "directory type char: {beta:?}"
    );

    let kilobyte = line_for(&out, "kilobyte.bin");
    assert_eq!(
        kilobyte[4], "2048",
        "size column must be raw bytes: {kilobyte:?}"
    );
}

#[test]
fn vdir_forwards_extra_args() {
    let dir = fixture();
    // -h (human-readable) must reach the ls engine, not vdir's own flags.
    let out = run_vdir(&dir, &["-h"]);
    let kilobyte = line_for(&out, "kilobyte.bin");
    assert_eq!(
        kilobyte[4], "2.0K",
        "-h must enable human-readable sizes: {kilobyte:?}"
    );
    let alpha = line_for(&out, "alpha.txt");
    assert_eq!(alpha[4], "5", "sub-kilobyte sizes stay raw: {alpha:?}");
}

#[test]
fn vdir_almost_all_shows_hidden_entries() {
    let dir = fixture();
    fs::write(dir.path().join(".hidden"), b"x").expect("write hidden file");
    let plain = run_vdir(&dir, &[]);
    assert_eq!(plain.lines().count(), 3, "dotfiles hidden by default");
    let almost_all = run_vdir(&dir, &["-A"]);
    assert_eq!(almost_all.lines().count(), 4, "-A must reveal .hidden");
    assert!(
        !line_for(&almost_all, ".hidden").is_empty(),
        ".hidden must be listed with -A"
    );
}

#[test]
fn vdir_lists_multiple_targets_like_ls() {
    let dir = fixture();
    let sub = TempDir::new().expect("create second temp dir");
    fs::write(sub.path().join("solo.txt"), b"y").expect("write fixture file");

    let out = cargo_bin_cmd!("vdir")
        .arg(dir.path())
        .arg(sub.path())
        .output()
        .expect("run vdir");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    // With multiple targets ls -l prints per-directory headers and totals.
    assert!(
        text.contains(&format!("{}:", dir.path().display())),
        "header missing: {text}"
    );
    assert!(
        text.contains(&format!("{}:", sub.path().display())),
        "header missing: {text}"
    );
    assert!(text.contains("total "), "total line missing: {text}");
    assert!(
        text.contains("solo.txt"),
        "second target contents missing: {text}"
    );
}
