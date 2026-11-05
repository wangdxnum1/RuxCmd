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
fn default_overwrites_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    // GNU cp overwrites an existing destination by default.
    cargo_bin_cmd!("cp").arg(&a).arg(&b).assert().success();
    assert_eq!(fs::read_to_string(&b).unwrap(), "A");
}

#[test]
fn no_clobber_skips_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    cargo_bin_cmd!("cp")
        .arg("-n")
        .arg(&a)
        .arg(&b)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&a).unwrap(), "A", "source must survive");
    assert_eq!(
        fs::read_to_string(&b).unwrap(),
        "B",
        "destination must survive"
    );
}

#[test]
fn no_clobber_wins_over_force_in_both_orders() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();

    for flags in [["-n", "-f"], ["-f", "-n"]] {
        fs::write(&b, "B").unwrap();
        cargo_bin_cmd!("cp")
            .args(flags)
            .arg(&a)
            .arg(&b)
            .assert()
            .success();
        assert_eq!(
            fs::read_to_string(&b).unwrap(),
            "B",
            "-n must win over -f in order {flags:?}"
        );
    }
}

#[test]
fn last_interactive_flag_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();

    // -i comes last: prompt, answer y, overwrite.
    fs::write(&b, "B").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-n", "-i"])
        .arg(&a)
        .arg(&b)
        .write_stdin("y\n")
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&b).unwrap(), "A");

    // -n comes last: skip silently, stdin is not consumed.
    fs::write(&b, "B").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-i", "-n"])
        .arg(&a)
        .arg(&b)
        .write_stdin("y\n")
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&b).unwrap(), "B");
}

#[test]
fn read_only_destination_requires_force() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();
    let mut perm = fs::metadata(&b).unwrap().permissions();
    perm.set_readonly(true);
    fs::set_permissions(&b, perm).unwrap();

    // GNU cp needs -f to replace an unwritable destination.
    cargo_bin_cmd!("cp")
        .arg(&a)
        .arg(&b)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("Permission denied"));
    assert_eq!(fs::read_to_string(&b).unwrap(), "B");

    // Answering y at the -i prompt does not substitute for -f.
    cargo_bin_cmd!("cp")
        .arg("-i")
        .arg(&a)
        .arg(&b)
        .write_stdin("y\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("Permission denied"));
    assert_eq!(fs::read_to_string(&b).unwrap(), "B");

    // -f clears the read-only bit and replaces the file.
    cargo_bin_cmd!("cp")
        .arg("-f")
        .arg(&a)
        .arg(&b)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&b).unwrap(), "A");
}

#[test]
fn interactive_prompt_is_written_to_stderr() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    cargo_bin_cmd!("cp")
        .arg("-i")
        .arg(&a)
        .arg(&b)
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::contains("overwrite '"));
    assert_eq!(fs::read_to_string(&b).unwrap(), "A");
}

#[test]
fn recursive_conflict_exits_one_and_continues() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("locked.txt"), "NEW").unwrap();
    fs::write(src.join("plain.txt"), "NEW2").unwrap();

    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();
    fs::create_dir(out.join("src")).unwrap();
    let locked = out.join("src").join("locked.txt");
    fs::write(&locked, "OLD").unwrap();
    let mut perm = fs::metadata(&locked).unwrap().permissions();
    perm.set_readonly(true);
    fs::set_permissions(&locked, perm).unwrap();

    cargo_bin_cmd!("cp")
        .arg("-r")
        .arg(&src)
        .arg(&out)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("Permission denied"));

    // The failing entry is reported, but the rest of the tree still copies.
    assert_eq!(fs::read_to_string(&locked).unwrap(), "OLD");
    assert_eq!(
        fs::read_to_string(out.join("src").join("plain.txt")).unwrap(),
        "NEW2"
    );
}
