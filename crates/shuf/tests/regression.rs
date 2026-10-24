//! Regression tests for P0 #29 (Task 44): `--random-source` must seed the
//! shuffle from the contents of the referenced file (FNV over up to 64 KiB),
//! not from the argument string itself. The old implementation hashed the
//! path string, so `shuf --random-source=/dev/urandom` produced the same
//! permutation on every run.

use assert_cmd::cargo::cargo_bin_cmd;
use std::path::Path;

fn run_with_source(source: &Path) -> Vec<u8> {
    cargo_bin_cmd!("shuf")
        .arg("-i")
        .arg("1-100")
        .arg("--random-source")
        .arg(source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

/// Two runs against the same source file must produce identical output
/// (reproducibility, the GNU random-source contract this build approximates).
#[test]
fn same_source_file_is_reproducible() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("seed.bin");
    std::fs::write(&src, b"ruxcmd-shuf-source-bytes").unwrap();

    let first = run_with_source(&src);
    let second = run_with_source(&src);
    assert_eq!(first, second);
}

/// The seed must come from the file contents: two *different* paths holding
/// identical content must shuffle identically. The old code hashed the path
/// string, so different paths gave different permutations.
#[test]
fn seed_depends_on_content_not_path() {
    let dir = tempfile::tempdir().unwrap();
    let src_a = dir.path().join("seed-a.bin");
    let src_b = dir.path().join("seed-b.bin");
    std::fs::write(&src_a, b"identical source payload").unwrap();
    std::fs::write(&src_b, b"identical source payload").unwrap();

    assert_eq!(run_with_source(&src_a), run_with_source(&src_b));
}

/// Rewriting the same source path with different bytes must change the
/// permutation. The old code never read the file, so its output never
/// changed.
#[test]
fn changing_source_content_changes_output() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("seed.bin");
    std::fs::write(&src, b"first payload").unwrap();
    let first = run_with_source(&src);

    std::fs::write(&src, b"a completely different payload").unwrap();
    let second = run_with_source(&src);

    assert_ne!(first, second);
}

/// A source file that cannot be opened is a hard error: exit 1 with
/// `shuf: cannot open '<path>'` on stderr. The old code never opened the
/// file and exited 0.
#[test]
fn missing_source_file_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-seed.bin");

    cargo_bin_cmd!("shuf")
        .arg("-i")
        .arg("1-10")
        .arg("--random-source")
        .arg(&missing)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(format!(
            "shuf: cannot open '{}'",
            missing.display()
        )));
}

/// Without `--random-source` the command still shuffles (behavior unchanged).
#[test]
fn without_random_source_still_shuffles() {
    let out = cargo_bin_cmd!("shuf")
        .arg("-i")
        .arg("1-20")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let nums: Vec<i32> = text.lines().map(|l| l.trim().parse().unwrap()).collect();
    let mut sorted = nums.clone();
    sorted.sort();
    assert_eq!(nums.len(), 20);
    assert_eq!(sorted, (1..=20).collect::<Vec<i32>>());
}
