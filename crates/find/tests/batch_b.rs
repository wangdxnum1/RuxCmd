//! Behaviour tests for the batch-B additions: `-print0`, `-delete`, `-size`,
//! `-mtime`, `-type l` and the two `-exec` forms.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};

fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), vec![b'a'; 10]).unwrap();
    fs::write(dir.path().join("big.bin"), vec![b'b'; 5000]).unwrap();
    fs::write(dir.path().join("empty.bin"), []).unwrap();
    fs::write(dir.path().join("sub").join("nested.txt"), vec![b'n'; 10]).unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> (Vec<u8>, i32) {
    let output = cargo_bin_cmd!("find")
        .args(args)
        .current_dir(dir)
        .assert()
        .get_output()
        .clone();
    (output.stdout.clone(), output.status.code().unwrap_or(-1))
}

fn split_nul(stdout: &[u8]) -> Vec<String> {
    assert!(
        stdout.ends_with(b"\0"),
        "expected NUL terminator: {:?}",
        stdout
    );
    stdout
        .split(|b| *b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8(part.to_vec()).unwrap())
        .collect()
}

#[test]
fn print0_separates_records_with_nul() {
    let dir = tree();
    let (stdout, code) = run(dir.path(), &[".", "-name", "a.txt", "-print0"]);
    assert_eq!(code, 0);
    assert!(!stdout.contains(&b'\n'));
    let mut names = split_nul(&stdout);
    names.sort();
    assert_eq!(names, [".\\a.txt".to_string()].to_vec());
}

#[test]
fn size_units_and_bounds_match_gnu() {
    let dir = tree();
    // 5000 bytes rounds up to 5 KiB; 10-byte files are one 512-byte block.
    let (stdout, _) = run(dir.path(), &[".", "-name", "big.bin", "-size", "5k"]);
    assert!(String::from_utf8_lossy(&stdout).contains("big.bin"));

    let (stdout, _) = run(dir.path(), &[".", "-name", "big.bin", "-size", "+4k"]);
    assert!(String::from_utf8_lossy(&stdout).contains("big.bin"));

    let (stdout, _) = run(dir.path(), &[".", "-name", "big.bin", "-size", "6k"]);
    assert!(stdout.is_empty());

    let (stdout, _) = run(dir.path(), &[".", "-name", "empty.bin", "-size", "0c"]);
    assert!(String::from_utf8_lossy(&stdout).contains("empty.bin"));

    // 10 bytes is one default block, so `-size -1` excludes it.
    let (stdout, _) = run(dir.path(), &[".", "-name", "a.txt", "-size", "-1"]);
    assert!(stdout.is_empty());
}

#[test]
fn mtime_filters_by_whole_days() {
    let dir = tree();
    // Freshly written files are zero days old.
    let (stdout, _) = run(dir.path(), &[".", "-name", "a.txt", "-mtime", "-1"]);
    assert!(String::from_utf8_lossy(&stdout).contains("a.txt"));

    let (stdout, _) = run(dir.path(), &[".", "-name", "a.txt", "-mtime", "0"]);
    assert!(String::from_utf8_lossy(&stdout).contains("a.txt"));

    let (stdout, _) = run(dir.path(), &[".", "-name", "a.txt", "-mtime", "+0"]);
    assert!(stdout.is_empty());
}

#[test]
fn type_l_reports_reparse_points_without_following_them() {
    let dir = tree();
    let junction = dir.path().join("junction");
    // Junctions can be created without administrator rights; file symlinks
    // cannot, so a junction is the portable way to exercise -type l here.
    let status = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(&junction)
        .arg(dir.path().join("sub"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("spawn cmd");
    if !status.success() {
        eprintln!("skipping: cannot create a junction in this environment");
        return;
    }

    let (stdout, code) = run(dir.path(), &[".", "-type", "l"]);
    assert_eq!(code, 0);
    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("junction"), "stdout: {}", text);

    // `-type d` excludes the reparse point, and the walk must not descend
    // into it, so `nested.txt` never appears.
    let (stdout, _) = run(dir.path(), &[".", "-type", "d"]);
    let text = String::from_utf8_lossy(&stdout);
    assert!(!text.contains("junction"), "stdout: {}", text);

    let (stdout, _) = run(dir.path(), &[".", "-name", "nested.txt"]);
    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("sub\\nested.txt"), "stdout: {}", text);
    assert!(!text.contains("junction"), "stdout: {}", text);
}

#[test]
fn delete_removes_children_before_parents() {
    let dir = tree();
    let root = dir.path().join("doomed");
    fs::create_dir_all(root.join("inner")).unwrap();
    fs::write(root.join("inner").join("file.txt"), "x").unwrap();

    let (_, code) = run(dir.path(), &["doomed", "-delete"]);
    assert_eq!(code, 0);
    assert!(!root.exists());
}

#[test]
fn delete_honours_the_name_filter() {
    let dir = tree();
    let (_, code) = run(dir.path(), &[".", "-name", "empty.bin", "-delete"]);
    assert_eq!(code, 0);
    assert!(!dir.path().join("empty.bin").exists());
    assert!(dir.path().join("a.txt").exists());
}

#[test]
fn delete_runs_in_expression_order_with_print() {
    let dir = tree();
    let root = dir.path().join("doomed");
    fs::create_dir_all(root.join("inner")).unwrap();
    fs::write(root.join("inner").join("file.txt"), "x").unwrap();

    // -delete implies depth-first order, so the file is printed first.
    let (stdout, code) = run(dir.path(), &["doomed", "-print", "-delete"]);
    assert_eq!(code, 0);
    let text = String::from_utf8_lossy(&stdout);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "stdout: {}", text);
    assert!(lines[0].contains("file.txt"), "stdout: {}", text);
    assert!(lines[2].ends_with("doomed"), "stdout: {}", text);
    assert!(!root.exists());
}

#[test]
fn delete_runs_in_expression_order_with_print0_and_exec() {
    let dir = tree();
    let root = dir.path().join("doomed");
    fs::create_dir_all(root.join("inner")).unwrap();
    fs::write(root.join("inner").join("file.txt"), "x").unwrap();

    let (stdout, code) = run(dir.path(), &["doomed", "-print0", "-delete"]);
    assert_eq!(code, 0);
    assert!(!stdout.contains(&b'\n'));
    assert_eq!(split_nul(&stdout).len(), 3);
    assert!(!root.exists());

    // -delete before -exec: the command still receives the path afterwards.
    fs::create_dir_all(root.join("inner")).unwrap();
    fs::write(root.join("inner").join("file.txt"), "x").unwrap();
    let (stdout, code) = run(
        dir.path(),
        &["doomed", "-delete", "-exec", "cmd", "/c", "echo", "{}", ";"],
    );
    assert_eq!(code, 0);
    let text = String::from_utf8_lossy(&stdout);
    assert_eq!(text.lines().count(), 3, "stdout: {}", text);
    assert!(!root.exists());
}

#[test]
fn exec_runs_once_per_entry() {
    let dir = tree();
    let (stdout, code) = run(
        dir.path(),
        &[
            ".", "-name", "a.txt", "-exec", "cmd", "/c", "echo", "{}", ";",
        ],
    );
    assert_eq!(code, 0);
    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("a.txt"), "stdout: {}", text);
    assert_eq!(text.lines().count(), 1, "stdout: {}", text);
}

#[test]
fn exec_plus_batches_all_paths_into_one_call() {
    let dir = tree();
    let (stdout, code) = run(
        dir.path(),
        &[
            ".", "-name", "*.txt", "-exec", "cmd", "/c", "echo", "{}", "+",
        ],
    );
    assert_eq!(code, 0);
    let text = String::from_utf8_lossy(&stdout);
    // One invocation prints both paths on a single line.
    assert_eq!(text.lines().count(), 1, "stdout: {}", text);
    assert!(text.contains("a.txt"), "stdout: {}", text);
    assert!(text.contains("nested.txt"), "stdout: {}", text);
}

#[test]
fn exec_without_terminator_is_an_error() {
    let dir = tree();
    cargo_bin_cmd!("find")
        .args([".", "-exec", "cmd", "/c", "echo"])
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains("missing argument to '-exec'"));
}

#[test]
fn exec_plus_requires_trailing_placeholder() {
    let dir = tree();
    cargo_bin_cmd!("find")
        .args([".", "-exec", "cmd", "/c", "echo", "x", "+"])
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "requires '{}' as the last argument",
        ));
}

#[test]
fn exec_failure_makes_find_exit_nonzero() {
    let dir = tree();
    cargo_bin_cmd!("find")
        .args([
            ".", "-name", "a.txt", "-exec", "cmd", "/c", "exit", "3", ";",
        ])
        .current_dir(dir.path())
        .assert()
        .code(1);
}

#[test]
fn no_action_still_prints_by_default() {
    let dir = tree();
    let (stdout, code) = run(dir.path(), &[".", "-name", "a.txt"]);
    assert_eq!(code, 0);
    assert_eq!(
        String::from_utf8_lossy(&stdout).trim(),
        PathBuf::from(".\\a.txt").display().to_string()
    );
}
