//! Integration tests for yes: multi-operand joining and BrokenPipe handling.

use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};

fn spawn_yes(args: &[&str]) -> (Child, ChildStdout) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yes"))
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let out = child.stdout.take().unwrap();
    (child, out)
}

fn first_line(args: &[&str]) -> String {
    let (mut child, mut out) = spawn_yes(args);
    let mut buf = [0u8; 64];
    let n = out.read(&mut buf).unwrap();
    drop(out);
    let _ = child.wait();
    let s = String::from_utf8_lossy(&buf[..n]).to_string();
    s.lines().next().unwrap_or_default().to_string()
}

#[test]
fn repeats_y_by_default() {
    assert_eq!(first_line(&[]), "y");
}

#[test]
fn joins_multiple_operands() {
    assert_eq!(first_line(&["a", "b"]), "a b");
    assert_eq!(first_line(&["hello", "world", "x"]), "hello world x");
}

#[test]
fn broken_pipe_exits_zero() {
    let (mut child, out) = spawn_yes(&[]);
    // Close the read end; yes must notice BrokenPipe and exit 0 silently.
    drop(out);
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(0), "status: {:?}", status);
}
