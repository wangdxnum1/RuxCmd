//! Round 12 I1 tests: hunks parsed by declared counts (empty lines and
//! `---`-prefixed deletions must not truncate), multi-file patches apply
//! per `---`/`+++` section, and `.rej` files contain the rejected hunk's
//! original lines.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// A hunk body containing an empty context line (trailing-space-stripped
/// diff) must be parsed by the declared counts, not truncated at the empty
/// line.
#[test]
fn empty_context_line_does_not_truncate_hunk() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f.txt"), "a\n\nb\n").unwrap();
    let patch = dir.path().join("p.diff");
    // The empty context line appears as a bare empty line (no leading
    // space) — a common real-world form after whitespace stripping.
    fs::write(
        &patch,
        "--- f.txt\n+++ f.txt\n@@ -1,3 +1,4 @@\n a\n\n+new\n b\n",
    )
    .unwrap();

    cargo_bin_cmd!("patch")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        "a\n\nnew\nb\n"
    );
}

/// A deletion whose content starts with `--` produces a body line
/// `--- ...`; counting-based parsing must treat it as a deletion line,
/// not a new file header.
#[test]
fn deletion_of_dash_dash_line_is_not_a_file_header() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f.txt"), "one\n-- two\nthree\n").unwrap();
    let patch = dir.path().join("p.diff");
    fs::write(
        &patch,
        "--- f.txt\n+++ f.txt\n@@ -1,3 +1,2 @@\n one\n--- two\n three\n",
    )
    .unwrap();

    cargo_bin_cmd!("patch")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        "one\nthree\n"
    );
}

/// A multi-file patch applies each `---`/`+++` section to its own target.
#[test]
fn multi_file_patch_updates_both_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "alpha\n").unwrap();
    fs::write(dir.path().join("b.txt"), "bravo\n").unwrap();
    let patch = dir.path().join("p.diff");
    fs::write(
        &patch,
        "--- a.txt\n+++ a.txt\n@@ -1,1 +1,1 @@\n-alpha\n+ALPHA\n--- b.txt\n+++ b.txt\n@@ -1,1 +1,1 @@\n-bravo\n+BRAVO\n",
    )
    .unwrap();

    cargo_bin_cmd!("patch")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("a.txt")).unwrap(),
        "ALPHA\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("b.txt")).unwrap(),
        "BRAVO\n"
    );
}

/// A rejected hunk is written to `.rej` as its original lines (header and
/// body), not as an error message; hunks before it are still applied.
#[test]
fn reject_file_contains_original_hunk_lines() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f.txt"), "one\ntwo\nthree\n").unwrap();
    let patch = dir.path().join("p.diff");
    // First hunk applies; second hunk is well-formed ("gone" is simply
    // absent from the file, so its context check fails and it is rejected).
    fs::write(
        &patch,
        "--- f.txt\n+++ f.txt\n@@ -1,3 +1,3 @@\n one\n-two\n+TWO\n three\n@@ -3,1 +3,1 @@\n-gone\n+MISS\n",
    )
    .unwrap();

    cargo_bin_cmd!("patch")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .code(1);

    // Successfully applied hunks are written back even when others fail.
    assert_eq!(
        fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        "one\nTWO\nthree\n"
    );

    let rej = fs::read_to_string(dir.path().join("f.txt.rej")).unwrap();
    assert!(rej.contains("@@ -3,1 +3,1 @@"), "rej was: {}", rej);
    assert!(rej.contains("-gone"), "rej was: {}", rej);
    assert!(rej.contains("+MISS"), "rej was: {}", rej);
    assert!(
        !rej.contains("reject:") && !rej.contains("expected"),
        ".rej must contain hunk lines, not diagnostics; was: {}",
        rej
    );
}

/// With -f the patch still applies and rejects are recorded, but the run
/// reports success (historical --force semantics of this port).
#[test]
fn force_mode_reports_success_with_reject_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f.txt"), "one\n").unwrap();
    let patch = dir.path().join("p.diff");
    fs::write(
        &patch,
        "--- f.txt\n+++ f.txt\n@@ -1,1 +1,1 @@\n-gone\n+HERE\n",
    )
    .unwrap();

    cargo_bin_cmd!("patch")
        .arg("-f")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("f.txt.rej").exists());
}
