//! Behavior tests for fold: non-UTF-8 input must fold instead of erroring,
//! and `-b` must slice and emit pure bytes with no lossy rewriting
//! (Round 12 F1).

use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fold"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn fold");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for fold")
}

// --- P1: non-UTF-8 input must not abort with an encoding error -------------

/// Default (character) mode reads raw bytes; the invalid byte becomes U+FFFD
/// and folding continues. Pre-fix `read_line` errored and exited 1.
#[test]
fn non_utf8_input_is_folded_not_rejected() {
    let out = run(&["-w", "1"], b"a\xFFb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stderr, b"");
    assert_eq!(out.stdout, b"a\n\xef\xbf\xbd\nb\n");
}

// --- P1: -b must conserve the input bytes exactly --------------------------

/// `fold -b` slices on byte boundaries and writes the segments back
/// verbatim; pre-fix `from_utf8_lossy` turned every multi-byte or invalid
/// byte into U+FFFD, destroying the content.
#[test]
fn bytes_mode_conserves_input_bytes() {
    let out = run(&["-b", "-w", "2"], b"\xFF\xFEabc\n");
    assert_eq!(out.status.code(), Some(0));
    // Content bytes FF FE 61 62 63 are all preserved; only the fold
    // newlines are added.
    assert_eq!(out.stdout, b"\xff\xfe\nab\nc\n");
}
