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

/// `--max-depth=N` keeps the counting identical but prints only directory
/// lines up to N levels below the operand (the operand itself is depth 0).
/// Output paths use `/` separators like GNU.
#[test]
fn max_depth_limits_printed_directory_lines() {
    let dir = nested_tree();
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--max-depth=1"]).trim(),
        "1\ta/b\n1\ta"
    );
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--max-depth=0"]).trim(),
        "1\ta"
    );
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--max-depth=2"]).trim(),
        "1\ta/b/c\n1\ta/b\n1\ta"
    );
}

/// GNU parses the depth with `xstrtoumax(..., base 0)`: hex and leading
/// whitespace/plus are accepted, trailing junk, signs and overflow are not.
#[test]
fn max_depth_accepts_gnu_number_forms() {
    let dir = nested_tree();
    let full = "1\ta/b/c\n1\ta/b\n1\ta";
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--max-depth=0x10"]).trim(),
        full
    );
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--max-depth= 2"]).trim(),
        full
    );
    for bad in ["-1", "99999999999999999999", "2x"] {
        cargo_bin_cmd!("du")
            .arg(format!("--max-depth={bad}"))
            .arg("a")
            .current_dir(dir.path())
            .assert()
            .code(1)
            .stderr(predicates::str::contains("invalid maximum depth"));
    }
}

/// A file operand gets exactly one line with `-a`: the operand line itself,
/// not a second one from the nested-file path.
#[test]
fn all_file_operand_prints_one_line() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("size.bin"), vec![b'x'; 5000]).unwrap();
    let mut cmd = cargo_bin_cmd!("du");
    let out = cmd
        .args(["-a", "-b"])
        .arg("size.bin")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "5000\tsize.bin");
}

/// `-a` writes a line for every file in post-order position (readdir order,
/// before its parent directory's line).
#[test]
fn all_prints_file_lines_in_post_order() {
    let dir = nested_tree();
    assert_eq!(
        du_in(&dir, &["-a", "-b"]).trim(),
        "10\ta/b/c/z\n10\ta/b/c\n10\ta/b/y\n20\ta/b\n10\ta/x\n30\ta"
    );
}

/// With `--max-depth` files count as the depth of their directory level:
/// `-a --max-depth=0` shows only the operand, `-a --max-depth=1` shows the
/// operand's direct children of both kinds.
#[test]
fn all_with_max_depth_limits_files_like_dirs() {
    let dir = nested_tree();
    assert_eq!(
        du_in(&dir, &["-a", "--apparent-size", "--max-depth=0"]).trim(),
        "1\ta"
    );
    assert_eq!(
        du_in(&dir, &["-a", "--apparent-size", "--max-depth=1"]).trim(),
        "1\ta/b\n1\ta/x\n1\ta"
    );
}

/// GNU refuses `-a` together with `-s` (usage error, exit 1).
#[test]
fn all_and_summarize_are_rejected() {
    let dir = nested_tree();
    cargo_bin_cmd!("du")
        .args(["-a", "-s"])
        .arg("a")
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "cannot both summarize and show all entries",
        ));
}

/// `-s` with an explicit positive `--max-depth` is a usage failure; with
/// `--max-depth=0` GNU only warns and behaves like plain `-s`.
#[test]
fn summarize_conflicts_with_positive_max_depth() {
    let dir = nested_tree();
    cargo_bin_cmd!("du")
        .args(["-s", "--max-depth=1"])
        .arg("a")
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains(
            "summarizing conflicts with --max-depth=1",
        ));
}

#[test]
fn summarize_with_zero_max_depth_warns_and_summarizes() {
    let dir = nested_tree();
    let out = cargo_bin_cmd!("du")
        .args(["-s", "--apparent-size", "--max-depth=0"])
        .arg("a")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "summarizing is the same as using --max-depth=0",
        ))
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap().trim(), "1\ta");
}

/// `--exclude` prunes matching directories (whole subtree, not counted) and
/// files, before their lines would be printed. Slash-bearing patterns match
/// any suffix of the path at component boundaries.
#[test]
fn exclude_prunes_matching_dirs_and_files() {
    let dir = nested_tree();
    // A pruned directory disappears from the listing; the operand shrinks.
    assert_eq!(
        du_in(&dir, &["--apparent-size", "--exclude=b"]).trim(),
        "1\ta"
    );
    // A file pattern removes it from the count and from the -a listing.
    assert_eq!(
        du_in(&dir, &["-a", "-b", "--exclude=y"]).trim(),
        "10\ta/b/c/z\n10\ta/b/c\n10\ta/b\n10\ta/x\n20\ta"
    );
    // `b/*` keeps directory b itself but empties it.
    assert_eq!(
        du_in(&dir, &["-b", "--exclude=b/*"]).trim(),
        "0\ta/b\n10\ta"
    );
    // Patterns are repeatable.
    assert_eq!(
        du_in(&dir, &["-b", "--exclude=x", "--exclude=z"]).trim(),
        "0\ta/b/c\n10\ta/b\n10\ta"
    );
}

/// A pattern without `/` matches any single path component.
#[test]
fn exclude_pattern_without_slash_matches_any_component() {
    let dir = nested_tree();
    assert_eq!(
        du_in(&dir, &["-b", "--exclude=y"]).trim(),
        "10\ta/b/c\n10\ta/b\n20\ta"
    );
}

/// An excluded operand produces no output at all and exits 0.
#[test]
fn excluded_operand_produces_no_output() {
    let dir = nested_tree();
    cargo_bin_cmd!("du")
        .args(["-b", "--exclude=a"])
        .arg("a")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

/// On a single volume `-x` changes nothing; the real cross-volume pruning is
/// covered by unit tests with an injected volume predicate.
#[test]
fn one_file_system_traverses_same_volume_normally() {
    let dir = nested_tree();
    assert_eq!(du_in(&dir, &["-b", "-a"]), du_in(&dir, &["-b", "-a", "-x"]));
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
