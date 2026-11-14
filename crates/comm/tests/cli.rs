use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::predicate;
use std::fs;

// --- sorted-input validation --------------------------------------------------

/// Unsorted input triggers GNU's file-specific and general diagnostics,
/// with exit code 1.
#[test]
fn unsorted_input_reports_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("f1"), "b\na\n").unwrap();
    fs::write(dir.path().join("f2"), "b\nc\n").unwrap();
    cargo_bin_cmd!("comm")
        .arg(dir.path().join("f1"))
        .arg(dir.path().join("f2"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "comm: file 1 is not in sorted order",
        ))
        .stderr(predicate::str::contains(
            "comm: input is not in sorted order",
        ));
}

/// Sorted input produces normal output with exit code 0 and no diagnostics.
#[test]
fn sorted_input_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("f1"), "a\nb\n").unwrap();
    fs::write(dir.path().join("f2"), "a\nc\n").unwrap();
    cargo_bin_cmd!("comm")
        .arg(dir.path().join("f1"))
        .arg(dir.path().join("f2"))
        .assert()
        .success()
        .stderr("");
}
