//! Regression tests for Task 12 (P0 #13 + adjacent P1):
//! - `.`/`..` sources with an existing directory destination used to panic
//!   on `source.file_name().unwrap()` (main.rs:104); they must now merge the
//!   source directory's *contents* into the destination.
//! - Multi-source copies to a non-directory target used to overwrite the
//!   target file once per source; they must be rejected up front.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn dot_source_merges_contents_into_existing_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(src.join("nested")).unwrap();
    fs::write(src.join("a.txt"), "alpha").unwrap();
    fs::write(src.join("nested").join("b.txt"), "beta").unwrap();

    let dst = tmp.path().join("dst");
    fs::create_dir(&dst).unwrap();
    fs::write(dst.join("keep.txt"), "keep").unwrap();

    // cwd = src, so the source operand is the literal `./.` form and the
    // destination (`../dst`) lies outside the copied tree.
    cargo_bin_cmd!("cp")
        .args(["-r", "./.", "../dst"])
        .current_dir(&src)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "alpha");
    assert_eq!(
        fs::read_to_string(dst.join("nested").join("b.txt")).unwrap(),
        "beta"
    );
    assert_eq!(fs::read_to_string(dst.join("keep.txt")).unwrap(), "keep");
    // Contents merged: no dst/src or dst/dst subdirectory may appear.
    assert!(!dst.join("src").exists());
    assert!(!dst.join("dst").exists());
}

#[test]
fn parent_dot_source_does_not_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let parent = tmp.path().join("parent");
    let inner = parent.join("inner");
    fs::create_dir_all(&inner).unwrap();
    fs::write(parent.join("data.txt"), "top").unwrap();
    fs::write(inner.join("nested.txt"), "deep").unwrap();

    // Destination outside `parent`, so `..` cannot self-recursively include it.
    let dst = tmp.path().join("dst");
    fs::create_dir(&dst).unwrap();

    // cwd = tmp/parent/inner, source `..` = tmp/parent, dest `../../dst` = tmp/dst.
    cargo_bin_cmd!("cp")
        .args(["-r", "..", "../../dst"])
        .current_dir(&inner)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst.join("data.txt")).unwrap(), "top");
    assert_eq!(
        fs::read_to_string(dst.join("inner").join("nested.txt")).unwrap(),
        "deep"
    );
}

#[test]
fn dot_source_requires_existing_directory_target() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("a.txt"), "A").unwrap();

    cargo_bin_cmd!("cp")
        .args(["-r"])
        .arg(".")
        .arg(tmp.path().join("no-such-dir"))
        .current_dir(&src)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("is not a directory"));
    assert!(!tmp.path().join("no-such-dir").exists());
}

#[test]
fn multi_source_rejects_non_directory_target() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    // Existing file target with -f: used to overwrite `c` once per source.
    let c = tmp.path().join("c.txt");
    fs::write(&c, "C").unwrap();
    cargo_bin_cmd!("cp")
        .arg("-f")
        .arg(&a)
        .arg(&b)
        .arg(&c)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("is not a directory"));
    assert_eq!(fs::read_to_string(&c).unwrap(), "C", "target must survive");

    // Missing target: must be refused as well, not written sequentially.
    cargo_bin_cmd!("cp")
        .arg(&a)
        .arg(&b)
        .arg(tmp.path().join("missing.txt"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("is not a directory"));
    assert!(!tmp.path().join("missing.txt").exists());

    // Multi-source into an existing directory keeps working.
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();
    cargo_bin_cmd!("cp")
        .arg(&a)
        .arg(&b)
        .arg(&out)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(out.join("b.txt")).unwrap(), "B");
}

#[test]
fn recursive_copy_into_existing_dir_keeps_source_name() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("f.txt"), "F").unwrap();
    fs::write(src.join("sub").join("n.txt"), "N").unwrap();

    let dst = tmp.path().join("dst");
    fs::create_dir(&dst).unwrap();

    cargo_bin_cmd!("cp")
        .arg("-r")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    // Existing behavior: plain dir source lands as dst/<name>.
    assert_eq!(
        fs::read_to_string(dst.join("src").join("f.txt")).unwrap(),
        "F"
    );
    assert_eq!(
        fs::read_to_string(dst.join("src").join("sub").join("n.txt")).unwrap(),
        "N"
    );
}

#[test]
fn default_refuses_overwriting_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    cargo_bin_cmd!("cp")
        .arg(&a)
        .arg(&b)
        .assert()
        .failure()
        .code(1);
    assert_eq!(fs::read_to_string(&b).unwrap(), "B");
}
