//! End-to-end behavior tests for the sync wrapper.
//!
//! The integration tests isolate the child from the developer's real PATH so
//! the external-command search and the rundll32 fallback are exercised
//! deterministically; the exit-code propagation logic itself is covered by
//! the unit tests in `src/main.rs` (injected `cmd /c exit N`).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const FALLBACK_NOTICE: &str = "no usable external sync";

#[test]
fn falls_back_with_notice_when_no_external_sync_on_path() {
    let empty_path = tempdir().unwrap();
    cargo_bin_cmd!("sync")
        .env("PATH", empty_path.path())
        .assert()
        .success()
        .stderr(predicates::str::contains(FALLBACK_NOTICE));
}

#[test]
fn does_not_recurse_when_path_points_at_copy_of_itself() {
    // Reproduces the P0 scenario: the install directory (holding our own
    // sync.exe) is on PATH. The old implementation resolved `sync` by bare
    // name and re-spawned itself without end; the fixed wrapper must run the
    // single copy, which detects itself and falls back, then exit 0.
    let path_dir = tempdir().unwrap();
    fs::copy(env!("CARGO_BIN_EXE_sync"), path_dir.path().join("sync.exe")).unwrap();

    cargo_bin_cmd!("sync")
        .env("PATH", path_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains(FALLBACK_NOTICE));
}
