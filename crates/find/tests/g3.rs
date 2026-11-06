//! Behaviour tests for the 0.4.6 additions: boolean operators (`-o`, `-a`,
//! `!`, `\( \)` groups) and the `-newer` / `-newermt` / `-newerat` tests,
//! verified against the local MSYS GNU find.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), vec![b'a'; 10]).unwrap();
    fs::write(dir.path().join("big.bin"), vec![b'b'; 5000]).unwrap();
    fs::write(dir.path().join("empty.bin"), []).unwrap();
    fs::write(dir.path().join("sub").join("nested.txt"), vec![b'n'; 10]).unwrap();
    dir
}

fn epoch(secs: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(secs as u64)
}

/// Forces both stamps of an existing file to fixed instants so `-newer`
/// comparisons never depend on test timing or filesystem timestamp granularity.
fn stamp(path: &Path, instant: SystemTime) {
    let file = File::options().write(true).open(path).unwrap();
    file.set_times(FileTimes::new().set_modified(instant).set_accessed(instant))
        .unwrap();
}

fn run(dir: &Path, args: &[&str]) -> (String, String, i32) {
    let output = cargo_bin_cmd!("find")
        .args(args)
        .current_dir(dir)
        .assert()
        .get_output()
        .clone();
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

fn sorted_lines(stdout: &str) -> Vec<String> {
    let mut lines: Vec<String> = stdout.lines().map(String::from).collect();
    lines.sort();
    lines
}

// --- boolean operators -------------------------------------------------------

#[test]
fn parenthesized_or_group_prints_both_branches() {
    let dir = tree();
    let (stdout, stderr, code) = run(
        dir.path(),
        &["(", "-name", "a.txt", "-o", "-name", "big.bin", ")"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    // The whole expression has no action, so GNU's implicit -print wraps it.
    assert_eq!(
        sorted_lines(&stdout),
        [".\\a.txt", ".\\big.bin"].map(String::from)
    );
}

#[test]
fn bang_and_not_negate_a_test() {
    let dir = tree();
    for form in [["!", "-name", "*.bin"], ["-not", "-name", "*.bin"]] {
        let (stdout, stderr, code) = run(dir.path(), &form);
        assert_eq!(code, 0, "stderr: {stderr}");
        let text = stdout.clone();
        assert!(text.contains("a.txt"), "{form:?}: {text}");
        assert!(text.contains("nested.txt"), "{form:?}: {text}");
        assert!(!text.contains("big.bin"), "{form:?}: {text}");
        assert!(!text.contains("empty.bin"), "{form:?}: {text}");
    }
}

#[test]
fn explicit_and_matches_implicit_and() {
    let dir = tree();
    // 5000 bytes rounds to 5k; the empty file has zero units, so only it is
    // "less than one unit".
    let (stdout, stderr, code) = run(dir.path(), &["-name", "*.bin", "-a", "-size", "-1k"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(sorted_lines(&stdout), [".\\empty.bin"].map(String::from));
}

#[test]
fn and_binds_tighter_than_or() {
    let dir = tree();
    // (-name *.bin -a -print) -o (-name a.txt): a.txt matches the right branch
    // but has no print there, while both .bin files are printed.
    let (stdout, stderr, code) = run(
        dir.path(),
        &["-name", "*.bin", "-print", "-o", "-name", "a.txt"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        sorted_lines(&stdout),
        [".\\big.bin", ".\\empty.bin"].map(String::from)
    );
}

#[test]
fn or_does_not_evaluate_a_true_left_side() {
    let dir = tree();
    // a.txt satisfies the left branch, so the -print on the right never runs.
    let (stdout, stderr, code) = run(dir.path(), &["(", "-name", "a.txt", "-o", "-print", ")"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let text = stdout.clone();
    assert!(!text.contains("a.txt"), "stdout: {text}");
    assert!(text.contains("big.bin"), "stdout: {text}");
    assert!(text.contains("nested.txt"), "stdout: {text}");
}

#[test]
fn and_does_not_evaluate_a_false_left_side() {
    let dir = tree();
    let doomed = dir.path().join("doomed");
    fs::create_dir_all(&doomed).unwrap();
    fs::write(doomed.join("file.txt"), "x").unwrap();

    // The name test fails for every entry, so -delete must never run.
    let (_, stderr, code) = run(dir.path(), &["doomed", "-name", "nope", "-delete"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(doomed.join("file.txt").exists());
}

#[test]
fn or_falls_through_to_the_right_action() {
    let dir = tree();
    let doomed = dir.path().join("doomed");
    fs::create_dir_all(&doomed).unwrap();
    fs::write(doomed.join("file.txt"), "x").unwrap();

    // file.txt fails the left branch, so the -delete on the right runs.
    let (_, stderr, code) = run(dir.path(), &["doomed", "-name", "nope", "-o", "-delete"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(!doomed.join("file.txt").exists());
}

#[test]
fn groups_combine_with_negation() {
    let dir = tree();
    // !(a.txt or *.bin) -print -> everything except the two named kinds.
    let (stdout, stderr, code) = run(
        dir.path(),
        &["!", "(", "-name", "a.txt", "-o", "-name", "*.bin", ")"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let text = stdout.clone();
    assert!(text.contains("nested.txt"), "stdout: {text}");
    assert!(text.contains("sub"), "stdout: {text}");
    assert!(!text.contains("a.txt"), "stdout: {text}");
    assert!(!text.contains("big.bin"), "stdout: {text}");
}

// --- expression parse errors (GNU wording, exit 1) ---------------------------

fn expect_error(dir: &Path, args: &[&str], needle: &str) {
    let (stdout, stderr, code) = run(dir, args);
    assert_eq!(code, 1, "args {args:?}: stdout: {stdout} stderr: {stderr}");
    assert!(
        stderr.contains(needle),
        "args {args:?}: expected {needle:?} in stderr: {stderr}"
    );
}

#[test]
fn dangling_operators_are_rejected() {
    let dir = tree();
    expect_error(dir.path(), &["!"], "expected an expression after '!'");
    expect_error(dir.path(), &["-not"], "expected an expression after '-not'");
    expect_error(
        dir.path(),
        &["-name", "x", "-a"],
        "expected an expression after '-a'",
    );
    expect_error(
        dir.path(),
        &["-name", "x", "-or"],
        "expected an expression after '-or'",
    );
    expect_error(
        dir.path(),
        &["-o", "-print"],
        "binary operator '-o' with nothing before it",
    );
    expect_error(
        dir.path(),
        &["!", "-a", "-print"],
        "binary operator '-a' with nothing before it",
    );
}

#[test]
fn unbalanced_parentheses_are_rejected() {
    let dir = tree();
    expect_error(dir.path(), &["(", "-name", "x"], "expecting to find a ')'");
    expect_error(dir.path(), &["(", ")"], "empty parentheses are not allowed");
    expect_error(dir.path(), &["-name", "x", ")"], "too many ')'");
    expect_error(
        dir.path(),
        &["(", "-name", "x", "-o", ")"],
        "expected an expression between '-o' and ')'",
    );
}

// --- -newer ------------------------------------------------------------------

fn newer_tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("old.dat"), "o").unwrap();
    fs::write(dir.path().join("ref.dat"), "r").unwrap();
    fs::write(dir.path().join("new.dat"), "n").unwrap();
    // 1990-01-01, 2001-09-09 and 2020-01-01, all in UTC.
    stamp(&dir.path().join("old.dat"), epoch(631_152_000));
    stamp(&dir.path().join("ref.dat"), epoch(1_000_000_000));
    stamp(&dir.path().join("new.dat"), epoch(1_577_836_800));
    dir
}

#[test]
fn newer_compares_against_the_reference_mtime() {
    let dir = newer_tree();
    let (stdout, stderr, code) = run(dir.path(), &[".", "-name", "*.dat", "-newer", "ref.dat"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(sorted_lines(&stdout), [".\\new.dat"].map(String::from));
}

#[test]
fn newer_is_strict_about_equal_timestamps() {
    let dir = newer_tree();
    let (stdout, _, code) = run(dir.path(), &[".", "-name", "ref.dat", "-newer", "ref.dat"]);
    assert_eq!(code, 0);
    assert!(stdout.is_empty(), "stdout: {stdout}");
}

#[test]
fn newer_reports_a_missing_reference() {
    let dir = newer_tree();
    let (stdout, stderr, code) = run(dir.path(), &[".", "-newer", "ghost.dat"]);
    assert_eq!(code, 1, "stdout: {stdout}");
    assert!(stdout.is_empty(), "stdout: {stdout}");
    assert!(stderr.contains("ghost.dat"), "stderr: {stderr}");
    assert!(
        stderr.contains("No such file or directory"),
        "stderr: {stderr}"
    );
}

#[test]
fn newer_inside_a_group_is_resolved_before_the_walk() {
    let dir = newer_tree();
    let (_, stderr, code) = run(
        dir.path(),
        &["(", "-name", "x", "-o", "-newer", "ghost.dat", ")"],
    );
    assert_eq!(code, 1);
    assert!(stderr.contains("ghost.dat"), "stderr: {stderr}");
}

// --- -newermt / -newerat ------------------------------------------------------

#[test]
fn newermt_compares_modification_time() {
    let dir = newer_tree();
    let (stdout, stderr, code) = run(
        dir.path(),
        &[".", "-name", "*.dat", "-newermt", "2010-06-15"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(sorted_lines(&stdout), [".\\new.dat"].map(String::from));

    // Every test file is newer than the epoch.
    let (stdout, stderr, code) = run(dir.path(), &[".", "-name", "*.dat", "-newermt", "@0"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        sorted_lines(&stdout),
        [".\\new.dat", ".\\old.dat", ".\\ref.dat"].map(String::from)
    );

    // Date with a time-of-day component.
    let (stdout, stderr, code) = run(
        dir.path(),
        &[".", "-name", "*.dat", "-newermt", "2010-06-15T12:00:30"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(sorted_lines(&stdout), [".\\new.dat"].map(String::from));
}

#[test]
fn newerat_compares_access_time() {
    let dir = newer_tree();
    // old.dat's atime was forced to 1990 alongside its mtime; new.dat was
    // created by this test, so its atime is current regardless of NTFS
    // last-access updates.
    let (stdout, stderr, code) = run(
        dir.path(),
        &[".", "-name", "*.dat", "-newerat", "2010-01-01"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(sorted_lines(&stdout), [".\\new.dat"].map(String::from));

    let (stdout, stderr, code) = run(
        dir.path(),
        &[".", "-name", "*.dat", "-newerat", "1980-01-01"],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        sorted_lines(&stdout),
        [".\\new.dat", ".\\old.dat", ".\\ref.dat"].map(String::from)
    );
}

#[test]
fn unparseable_dates_are_rejected() {
    let dir = newer_tree();
    for date in [
        "not-a-date",
        "2020-13-01",
        "2020-01-32",
        "2020-01-01T25:00",
        "@abc",
    ] {
        expect_error(
            dir.path(),
            &[".", "-newermt", date],
            "I cannot figure out how to interpret",
        );
        expect_error(
            dir.path(),
            &[".", "-newerat", date],
            "I cannot figure out how to interpret",
        );
    }
}

#[test]
fn newer_family_requires_its_argument() {
    let dir = newer_tree();
    expect_error(dir.path(), &[".", "-newer"], "missing argument to '-newer'");
    expect_error(dir.path(), &[".", "-newermt"], "needs an argument");
    expect_error(dir.path(), &[".", "-newerat"], "needs an argument");
}

// --- combinations with existing behaviour -------------------------------------

#[test]
fn print0_after_a_group_works() {
    let dir = tree();
    let (stdout, stderr, code) = run(
        dir.path(),
        &[
            "(", "-name", "a.txt", "-o", "-name", "big.bin", ")", "-print0",
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(!stdout.contains('\n'));
    let names: Vec<String> = stdout
        .split('\0')
        .filter(|part| !part.is_empty())
        .map(String::from)
        .collect();
    assert_eq!(names.len(), 2, "stdout: {stdout:?}");
    assert!(names.contains(&".\\a.txt".to_string()));
    assert!(names.contains(&".\\big.bin".to_string()));
}

#[test]
fn exec_inside_a_group_runs_for_matches_only() {
    let dir = tree();
    let (stdout, stderr, code) = run(
        dir.path(),
        &[
            "(", "-name", "a.txt", "-o", "-name", "big.bin", ")", "-exec", "cmd", "/c", "echo",
            "{}", ";",
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let text = stdout.clone();
    assert!(text.contains("a.txt"), "stdout: {text}");
    assert!(text.contains("big.bin"), "stdout: {text}");
    assert!(!text.contains("nested.txt"), "stdout: {text}");
}

#[test]
fn regression_delete_with_name_after_delete() {
    // Expression order means -delete runs before -name is even seen: GNU
    // removes everything under the start point here, start point included.
    let dir = tree();
    let root = dir.path().join("doomed");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("empty.bin"), []).unwrap();
    fs::write(root.join("a.txt"), "x").unwrap();

    let (_, stderr, code) = run(dir.path(), &["doomed", "-delete", "-name", "empty.bin"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(!root.exists());
}

#[test]
fn regression_print_delete_order_in_groups() {
    let dir = tree();
    let root = dir.path().join("doomed");
    fs::create_dir_all(root.join("inner")).unwrap();
    fs::write(root.join("inner").join("file.txt"), "x").unwrap();

    let (stdout, stderr, code) = run(dir.path(), &["doomed", "-print", "-delete"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "stdout: {stdout}");
    assert!(lines[0].contains("file.txt"), "stdout: {stdout}");
    assert!(lines[2].ends_with("doomed"), "stdout: {stdout}");
    assert!(!root.exists());
}

#[test]
fn regression_paths_may_precede_the_expression() {
    let dir = tree();
    let (stdout, stderr, code) = run(dir.path(), &[".", "-name", "a.txt"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        sorted_lines(&stdout),
        [PathBuf::from(".\\a.txt").display().to_string()].to_vec()
    );
}
