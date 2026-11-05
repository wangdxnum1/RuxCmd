//! Regression tests for P0 #45 (Task 45) plus the GNU block/apparent-size
//! semantics: `du` reports 1K blocks of allocated space by default, `-b`
//! reports apparent bytes, `--apparent-size` counts logical sizes, `-h`
//! uses GNU wording, and hard links are counted once.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

/// Builds `a/x` (10 B), `a/b/y` (10 B), `a/b/c/z` (10 B) — 30 bytes of file
/// content under the argument directory `a`.
fn nested_tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("a");
    fs::create_dir_all(root.join("b").join("c")).unwrap();
    fs::write(root.join("x"), vec![b'x'; 10]).unwrap();
    fs::write(root.join("b").join("y"), vec![b'y'; 10]).unwrap();
    fs::write(root.join("b").join("c").join("z"), vec![b'z'; 10]).unwrap();
    dir
}

fn du_in(dir: &tempfile::TempDir, extra: &[&str]) -> String {
    let mut cmd = cargo_bin_cmd!("du");
    for a in extra {
        cmd.arg(a);
    }
    let out = cmd
        .arg("a")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).unwrap()
}

/// `du -s -b a` reports the recursive apparent total in bytes.
#[test]
fn summarize_reports_file_total_in_bytes_mode() {
    let dir = nested_tree();
    assert_eq!(du_in(&dir, &["-s", "-b"]).trim(), "30\ta");
}

/// Default mode reports 1K blocks of the apparent size here; GNU's
/// per-subdirectory listing stays a declared deviation, so one line only.
#[test]
fn default_mode_reports_1k_blocks() {
    let dir = nested_tree();
    assert_eq!(du_in(&dir, &["-s", "--apparent-size"]).trim(), "1\ta");
}

/// `-b` prints exact bytes rather than rounded blocks.
#[test]
fn bytes_mode_reports_exact_bytes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("size.bin"), vec![b'x'; 5000]).unwrap();
    let mut cmd = cargo_bin_cmd!("du");
    let out = cmd
        .arg("-b")
        .arg("size.bin")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "5000\tsize.bin");
}

/// `-h` follows GNU's numbering: one decimal below 10 units, integers above.
#[test]
fn human_readable_matches_gnu_wording() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("one_k.bin"), vec![b'x'; 1024]).unwrap();
    fs::write(dir.path().join("five_k.bin"), vec![b'x'; 5000]).unwrap();

    let mut got = Vec::new();
    for name in ["one_k.bin", "five_k.bin"] {
        let out = cargo_bin_cmd!("du")
            .args(["-h", "--apparent-size"])
            .arg(name)
            .current_dir(dir.path())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        got.push(String::from_utf8(out).unwrap().trim().to_string());
    }
    assert_eq!(got[0], "1.0K\tone_k.bin");
    assert_eq!(got[1], "4.9K\tfive_k.bin");
}

/// A hard-linked file contributes once, not once per directory entry.
#[test]
fn hard_link_is_counted_once() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("a.bin");
    fs::write(&original, vec![b'x'; 5000]).unwrap();
    fs::hard_link(&original, dir.path().join("b.bin")).unwrap();

    let out = cargo_bin_cmd!("du")
        .args(["-s", "-b", "."])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "5000\t.");
}

/// GNU omits the line for an argument that repeats an already-counted hard
/// link instead of printing a zero.
#[test]
fn duplicate_hard_link_argument_is_omitted() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("a.bin");
    fs::write(&original, vec![b'x'; 5000]).unwrap();
    fs::hard_link(&original, dir.path().join("b.bin")).unwrap();

    let out = cargo_bin_cmd!("du")
        .args(["-b", "a.bin", "b.bin"])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "5000\ta.bin");
}

/// An empty file has an apparent size of zero.
#[test]
fn empty_file_is_zero_bytes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("zero.bin"), []).unwrap();
    let out = cargo_bin_cmd!("du")
        .arg("-b")
        .arg("zero.bin")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "0\tzero.bin");
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
        .args(["-s", "-b", "a"])
        .arg("no-such-path-ruxcmd-du")
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::contains("30\ta"))
        .stderr(predicates::str::contains("du:"));
}
