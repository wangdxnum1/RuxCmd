//! Regression tests for P0 #33: `zip` must quote file names and the
//! destination before embedding them in the PowerShell `Compress-Archive`
//! command string. Names with spaces previously broke the command outright,
//! and names containing `'`, `;` or `$()` could inject arbitrary PowerShell
//! code. These tests shell out to the real `powershell` locally (no network).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

/// Archiving files whose names contain spaces, single quotes and PowerShell
/// metacharacters must succeed, and the resulting archive must contain one
/// entry per input file. Before the fix the unquoted names were spliced into
/// the command string, so `zip out.zip "my file.txt"` failed.
#[test]
fn archives_files_with_spaces_quotes_and_metacharacters() {
    let dir = tempdir().unwrap();
    // All characters below are legal in NTFS file names; `; $() & %` and `'`
    // would previously terminate or inject into the PowerShell command.
    let names = ["my file.txt", "it's a file.txt", "a;b $(x) & c%.txt"];
    for name in names {
        fs::write(dir.path().join(name), b"payload").unwrap();
    }

    // The destination name also contains a space.
    let out = dir.path().join("my out.zip");
    let mut cmd = cargo_bin_cmd!("zip");
    cmd.current_dir(dir.path()).arg("my out.zip");
    for name in names {
        cmd.arg(name);
    }
    cmd.assert().success();

    let bytes = fs::read(&out).unwrap();
    assert!(!bytes.is_empty(), "archive was not created");
    for name in names {
        // Entry names are stored literally in the local headers, so each one
        // must appear in the archive bytes. (File contents are deflate-
        // compressed, so the payload itself is not searched for here.)
        assert!(
            bytes.windows(name.len()).any(|w| w == name.as_bytes()),
            "entry name {name:?} missing from archive"
        );
    }
}
