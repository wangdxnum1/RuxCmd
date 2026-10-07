//! Regression tests for P0 #50: `-p/--strip` must strip leading path
//! components from the `+++` header (git-style `b/file.txt` names), and
//! `@@ -0,0` creation hunks must not underflow (`old_start - 1`).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// Git-style unified diff against `file.txt` with `a/` and `b/` prefixes.
const GIT_STYLE_DIFF: &str = "\
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,3 @@
 one
-two
+TWO
 three
";

/// Git-style new-file diff: `/dev/null` source, two added lines.
const NEW_FILE_DIFF: &str = "\
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+alpha
+beta
";

/// `-p1` strips the `b/` prefix so the patch lands on the plain `file.txt`
/// sitting in the working directory.
#[test]
fn strip_1_applies_git_style_diff_to_plain_name() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("file.txt"), "one\ntwo\nthree\n").unwrap();
    let patch = dir.path().join("git.diff");
    fs::write(&patch, GIT_STYLE_DIFF).unwrap();

    cargo_bin_cmd!("patch")
        .arg("-p1")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("file.txt")).unwrap(),
        "one\nTWO\nthree\n"
    );
}

/// `-p0` keeps the prefix, so `b/file.txt` does not exist and the run must
/// fail with exit 1 while the real file stays untouched.
#[test]
fn strip_0_keeps_prefix_and_fails_when_missing() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("file.txt"), "one\ntwo\nthree\n").unwrap();
    let patch = dir.path().join("git.diff");
    fs::write(&patch, GIT_STYLE_DIFF).unwrap();

    cargo_bin_cmd!("patch")
        .arg("-p0")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains("b/file.txt"));

    assert_eq!(
        fs::read_to_string(dir.path().join("file.txt")).unwrap(),
        "one\ntwo\nthree\n",
        "-p0 run must not modify the unprefixed file"
    );
}

/// A `/dev/null` creation diff with `-p1` must create `new.txt` containing
/// the two added lines instead of failing on the missing target.
#[test]
fn new_file_diff_creates_target_file() {
    let dir = TempDir::new().unwrap();
    let patch = dir.path().join("new.diff");
    fs::write(&patch, NEW_FILE_DIFF).unwrap();

    cargo_bin_cmd!("patch")
        .arg("--strip=1")
        .arg(&patch)
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("new.txt")).unwrap(),
        "alpha\nbeta\n"
    );
}

/// Direct regression for the `hunk.old_start - 1` underflow: a `@@ -0,0`
/// creation hunk applied to an existing empty file must succeed (it used to
/// panic in debug builds and always fail in release).
#[test]
fn creation_hunk_on_existing_empty_file_does_not_underflow() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    let patch = dir.path().join("create.diff");
    fs::write(&patch, "--- empty.txt\n+++ empty.txt\n@@ -0,0 +1,2 @@\n+alpha\n+beta\n").unwrap();

    cargo_bin_cmd!("patch")
        .arg(&patch)
        .arg("empty.txt")
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("empty.txt")).unwrap(),
        "alpha\nbeta\n"
    );
}
