//! Regression tests for the long-format (-l) rendering pipeline.
//!
//! P0 #5: `print_long` used to look up the raw entry by comparing the
//! *rendered* display name (which may contain ANSI color codes, `-Q` quotes
//! or `-F` indicators) against the plain entry name. With colors enabled the
//! match failed for directories/symlinks/executables, silently dropping the
//! `-i` inode column and the `-s` block column.
//!
//! The test forces coloring the way a real terminal would: `--color=always`
//! opens the ls-side gate, `CLICOLOR_FORCE=1` opens the `colored` crate's
//! gate (stdout is a pipe under assert_cmd, so `auto` alone never colors).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// Build a temp dir containing one plain file and one subdirectory.
/// The subdirectory is essential: its name gets ANSI codes (blue + bold),
/// which is what used to break the name-based entry lookup.
fn fixture() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("alpha.txt"), b"hi").expect("write fixture file");
    fs::create_dir(dir.path().join("beta")).expect("create fixture dir");
    dir
}

fn run_ls(dir: &TempDir, args: &[&str], force_color_env: bool) -> Vec<String> {
    let mut cmd = cargo_bin_cmd!("ls");
    cmd.args(args).arg(dir.path());
    if force_color_env {
        cmd.env("CLICOLOR_FORCE", "1");
    }
    let output = cmd.output().expect("run ls");
    assert!(
        output.status.success(),
        "ls failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn assert_inode_token(line: &str, token: &str) {
    assert!(
        token == "?" || token.bytes().all(|b| b.is_ascii_digit()),
        "inode column missing or malformed (expected digits or '?'): {line:?}"
    );
}

fn assert_permission_token(line: &str, token: &str) {
    assert_eq!(token.len(), 10, "permissions field malformed: {line:?}");
    assert!(
        matches!(token.chars().next(), Some('-') | Some('d') | Some('l')),
        "permissions field must start with file type char: {line:?}"
    );
}

#[test]
fn inode_column_survives_colored_names() {
    let dir = fixture();
    let lines = run_ls(&dir, &["-li", "--color=always"], true);
    assert_eq!(lines.len(), 2, "one line per entry expected: {lines:?}");
    for line in &lines {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        assert!(tokens.len() >= 2, "long-format line truncated: {line:?}");
        assert_inode_token(line, tokens[0]);
        assert_permission_token(line, tokens[1]);
    }
}

#[test]
fn block_column_survives_colored_names() {
    let dir = fixture();
    // -l -i -s: inode column, block column, then the long fields.
    let lines = run_ls(&dir, &["-lis", "--color=always"], true);
    assert_eq!(lines.len(), 2, "one line per entry expected: {lines:?}");
    for line in &lines {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        assert!(tokens.len() >= 3, "long-format line truncated: {line:?}");
        assert_inode_token(line, tokens[0]);
        assert!(
            tokens[1].bytes().all(|b| b.is_ascii_digit()),
            "block count column missing or malformed: {line:?}"
        );
        assert_permission_token(line, tokens[2]);
    }
}

#[test]
fn long_format_leading_fields_are_complete() {
    let dir = fixture();
    let lines = run_ls(&dir, &["-l", "--color=never"], false);
    assert_eq!(lines.len(), 2, "one line per entry expected: {lines:?}");
    for line in &lines {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        // perms nlink owner group size month day time name
        assert!(tokens.len() >= 9, "long-format line truncated: {line:?}");
        assert_permission_token(line, tokens[0]);
        assert!(
            tokens[1].bytes().all(|b| b.is_ascii_digit()),
            "nlink field must be numeric: {line:?}"
        );
        assert!(
            tokens[4].bytes().all(|b| b.is_ascii_digit()),
            "size field must be numeric: {line:?}"
        );
        let name = tokens[tokens.len() - 1];
        assert!(
            name == "alpha.txt" || name == "beta",
            "unexpected trailing token {name:?} in: {line:?}"
        );
    }
}
