//! Regression tests for P0 #26 (Task 41):
//! 1. The homemade `PATTERNN` repeat syntax (e.g. `/pat/3`) is removed; it
//!    used to drop the matched line from every output file. The only
//!    supported forms are `/substring/` and a line number — GNU `/re/+N`
//!    offsets and `{N}` repeats now report an explicit error.
//! 2. `--suffix-format` was a fake declaration (accepted, ignored) — the
//!    option no longer exists and must be rejected as unknown.
//! 3. A pattern with no match must fail with exit 1 (GNU behavior);
//!    without -k output files are removed, with -k they are kept.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

fn csplit_cmd() -> assert_cmd::Command {
    cargo_bin_cmd!("csplit")
}

/// `/PAT/` starts a new file *at* the matched line: the matched line itself
/// must survive in the following output file.
#[test]
fn matched_line_is_kept_in_next_output_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"x\nPAT\ny\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/PAT/"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(std::fs::read(dir.path().join("out.00")).unwrap(), b"x\n");
    assert_eq!(
        std::fs::read(dir.path().join("out.01")).unwrap(),
        b"PAT\ny\n",
        "matched line was dropped"
    );
}

/// Repeated patterns via line numbers (the GNU-portable substitute for the
/// removed homemade repeat syntax): every input line lands in exactly one
/// output file.
#[test]
fn repeated_line_number_patterns_keep_all_lines() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"l1\nl2\nl3\nl4\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "2", "3"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert_eq!(std::fs::read(dir.path().join("out.00")).unwrap(), b"l1\n");
    assert_eq!(std::fs::read(dir.path().join("out.01")).unwrap(), b"l2\n");
    assert_eq!(
        std::fs::read(dir.path().join("out.02")).unwrap(),
        b"l3\nl4\n"
    );
}

/// GNU offset syntax `/re/+3` was silently swallowed by the old repeat-tail
/// parser; it must now be rejected loudly.
#[test]
fn offset_pattern_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/a/+3"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("invalid pattern '/a/+3'"));

    assert!(
        !dir.path().join("out.00").exists(),
        "output files must be removed on error without -k"
    );
}

/// Negative offsets are rejected the same way.
#[test]
fn negative_offset_pattern_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/a/-1"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("invalid pattern '/a/-1'"));
}

/// The homemade `pattern+digit` repeat syntax is gone; `/pat/3` no longer
/// runs (and no longer drops matched lines) — it is an invalid pattern.
#[test]
fn homemade_repeat_syntax_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nPAT\nb\nPAT\nc\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/PAT/2"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("invalid pattern '/PAT/2'"));
}

/// GNU `{N}` repeats are not supported (never declared); they must be an
/// explicit error rather than a silent no-op.
#[test]
fn brace_repeat_pattern_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nPAT\nb\nPAT\nc\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/PAT/", "/PAT/{2}"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("invalid pattern '/PAT/{2}'"));
}

/// `--suffix-format` was accepted and ignored; it is no longer declared and
/// must fail as an unknown argument.
#[test]
fn suffix_format_is_rejected_as_unknown() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    csplit_cmd()
        .args(["--suffix-format=%03d", "in.txt", "--", "/a/"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("unexpected argument"));
}

/// GNU csplit fails on an unmatched pattern; without -k output files are
/// removed and the exit status is 1.
#[test]
fn unmatched_pattern_fails_and_removes_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    csplit_cmd()
        .args(["-s", "-f", "out.", "in.txt", "--", "/zzz/"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("match not found"));

    assert!(
        !dir.path().join("out.00").exists(),
        "partial output must be removed without -k"
    );
}

/// With -k the partially written files survive the match-not-found error.
#[test]
fn unmatched_pattern_keeps_files_with_k() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.txt"), b"a\nb\n").unwrap();

    csplit_cmd()
        .args(["-s", "-k", "-f", "out.", "in.txt", "--", "/zzz/"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("match not found"));

    assert_eq!(
        std::fs::read(dir.path().join("out.00")).unwrap(),
        b"a\nb\n",
        "remaining input must be flushed with -k"
    );
}
