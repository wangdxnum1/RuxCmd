//! Regression tests for P0 #45 (Task 45): `du -s <dir>` must still recurse
//! and report the summed byte total of the tree. The old code passed
//! `recursive=false` under `-s`, so only the directory's own
//! `metadata.len()` was reported (0 on NTFS). Unreadable paths must also
//! make the command exit 1 instead of 0.

use assert_cmd::cargo::cargo_bin_cmd;

/// Builds `a/x` (10 B), `a/b/y` (10 B), `a/b/c/z` (10 B) — 30 bytes of file
/// content under the argument directory `a`.
fn nested_tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("a");
    std::fs::create_dir_all(root.join("b").join("c")).unwrap();
    std::fs::write(root.join("x"), vec![b'x'; 10]).unwrap();
    std::fs::write(root.join("b").join("y"), vec![b'y'; 10]).unwrap();
    std::fs::write(root.join("b").join("c").join("z"), vec![b'z'; 10]).unwrap();
    dir
}

fn du_in(dir: &tempfile::TempDir, extra: &[&str]) -> Vec<u8> {
    let mut cmd = cargo_bin_cmd!("du");
    for a in extra {
        cmd.arg(a);
    }
    cmd.arg("a")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

/// `du -s a` must report the recursive total (30 bytes) as a single line.
/// The old code returned only the directory's own metadata length.
#[test]
fn summarize_recurses_and_reports_file_total() {
    let dir = nested_tree();
    let out = du_in(&dir, &["-s"]);
    assert_eq!(String::from_utf8(out).unwrap().trim(), "30 a");
}

/// Default mode (no `-s`) also prints exactly one line — the argument
/// line with the same recursive total (GNU's per-subdirectory listing is
/// a declared P2 deviation and stays out of scope).
#[test]
fn default_mode_prints_single_argument_line() {
    let dir = nested_tree();
    let out = du_in(&dir, &[]);
    assert_eq!(String::from_utf8(out).unwrap().trim(), "30 a");
}

/// A path that cannot be read must be reported on stderr and must make du
/// exit 1 (the old code always exited 0).
#[test]
fn unreadable_path_reports_error_and_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("du")
        .arg(dir.path().join("no-such-path-ruxcmd-du"))
        .assert()
        .code(1)
        .stderr(predicates::str::contains("du:"));
}

/// A bad path alongside a good one: the good path still gets its line,
/// the bad one is reported, and the command exits 1.
#[test]
fn bad_path_does_not_hide_good_path() {
    let dir = nested_tree();
    cargo_bin_cmd!("du")
        .arg("a")
        .arg("no-such-path-ruxcmd-du")
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::contains("30 a"))
        .stderr(predicates::str::contains("du:"));
}
