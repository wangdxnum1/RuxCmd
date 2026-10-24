//! Behavior tests for join: GNU output shape (key + file1 remaining fields +
//! file2 remaining fields) and the default whitespace field separator
//! (Round 12 F1).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_join"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn join");
    child
        .stdin
        .take()
        .expect("stdin pipe")
        .write_all(b"")
        .expect("failed to close stdin");
    child.wait_with_output().expect("failed to wait for join")
}

fn temp_file(tag: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("ruxcmd-join-{}-{}.txt", tag, std::process::id()));
    std::fs::write(&path, contents).expect("failed to write fixture");
    path
}

// --- P1: output shape must be key + remaining fields, no repeated key ------

/// file1 `A B`, file2 `A C` joined on the key `A` must print `A B C`; pre-fix
/// the output was `A B A C` (whole lines glued, file2's key repeated).
#[test]
fn output_shape_is_key_plus_remaining_fields() {
    let f1 = temp_file("shape1", "A B\n");
    let f2 = temp_file("shape2", "A C\n");

    let out = run(&[
        "-t",
        " ",
        f1.to_string_lossy().as_ref(),
        f2.to_string_lossy().as_ref(),
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"A B C\n");
}

// --- P1: without -t the default separator is blank runs, not TAB -----------

/// Space-separated files must join without `-t` (GNU splits on whitespace
/// runs and emits fields separated by single spaces); pre-fix the default
/// was TAB and space-separated input never matched.
#[test]
fn whitespace_default_joins_space_separated_files() {
    let f1 = temp_file("ws1", "1 x\n2 y\n");
    let f2 = temp_file("ws2", "1 p\n2 q\n");

    let out = run(&[f1.to_string_lossy().as_ref(), f2.to_string_lossy().as_ref()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"1 x p\n2 y q\n");
}
