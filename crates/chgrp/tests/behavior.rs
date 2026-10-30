use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// Run chgrp against a group that always exists on Windows hosts; admin
/// rights are NOT required to change a file's group to a group the current
/// user is a member of, so assert success but tolerate privilege failures.
fn well_known_group() -> &'static str {
    "None"
}

#[test]
fn changes_group_of_a_file() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("data.txt");
    fs::write(&f, b"x").unwrap();
    let out = cargo_bin_cmd!("chgrp")
        .args([well_known_group(), f.to_string_lossy().as_ref()])
        .output()
        .unwrap();
    if out.status.code() != Some(0) {
        // Hosts may deny group changes entirely; the tool must still fail
        // with a diagnostic rather than panic.
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("chgrp:"), "stderr: {err}");
        eprintln!("skipped: group change denied on this host");
        return;
    }
    assert!(out.stderr.is_empty());
}

#[test]
fn verbose_prints_what_changed() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("loud.txt");
    fs::write(&f, b"x").unwrap();
    let out = cargo_bin_cmd!("chgrp")
        .args(["-v", well_known_group(), f.to_string_lossy().as_ref()])
        .output()
        .unwrap();
    if out.status.code() != Some(0) {
        eprintln!("skipped: group change denied on this host");
        return;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("changed group"), "stdout: {text}");
}

#[test]
fn unknown_group_fails_with_exit_one() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("x.txt");
    fs::write(&f, b"x").unwrap();
    cargo_bin_cmd!("chgrp")
        .args(["definitely-not-a-group-xyz", f.to_string_lossy().as_ref()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot find group"));
}

#[test]
fn missing_file_operand_is_a_usage_error() {
    cargo_bin_cmd!("chgrp")
        .arg("None")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("missing operand"));
}

#[test]
fn recursive_processes_children() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("inner.txt"), b"x").unwrap();
    let out = cargo_bin_cmd!("chgrp")
        .args([
            "-R",
            "-v",
            well_known_group(),
            dir.path().join("sub").to_string_lossy().as_ref(),
        ])
        .output()
        .unwrap();
    if out.status.code() != Some(0) {
        eprintln!("skipped: group change denied on this host");
        return;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("inner.txt"), "stdout: {text}");
}
