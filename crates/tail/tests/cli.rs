//! Behavior tests for tail: stdin support, the `-n 0 -f` follow start
//! offset, and multi-file handling (P0 #23).
//!
//! Follow expectations verified against GNU coreutils 8.32 (`tail -f`,
//! `tail -n 0 -f`): the first `==> name <==` header has no leading blank
//! line, later headers are preceded by a blank line, and a header is
//! reprinted only when the displayed file changes; `tail -n 0 -f`
//! produces no initial output at all.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ruxcmd-tail-{}-{}.txt", tag, std::process::id()))
}

fn run_tail(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tail"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn tail");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for tail")
}

fn spawn_tail(args: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_tail"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn tail")
}

/// Reads exactly `n` bytes from the child's stdout, killing the child and
/// failing the test if the bytes do not arrive within `timeout`. The
/// stdout handle is put back so it can be read again later.
fn read_exact_with_deadline(child: &mut Child, n: usize, timeout: Duration) -> Vec<u8> {
    let mut stdout = child.stdout.take().expect("tail stdout must be piped");
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut buf = vec![0u8; n];
        let mut filled = 0usize;
        let mut err = None;
        while filled < n {
            match stdout.read(&mut buf[filled..]) {
                Ok(0) => break, // child closed stdout
                Ok(k) => filled += k,
                Err(e) => {
                    err = Some(e.to_string());
                    break;
                }
            }
        }
        buf.truncate(filled);
        let _ = tx.send((buf, err, stdout));
    });
    match rx.recv_timeout(timeout) {
        Ok((buf, err, stdout)) => {
            child.stdout = Some(stdout);
            if let Some(e) = err {
                panic!("error reading tail stdout: {}", e);
            }
            buf
        }
        Err(_) => {
            let _ = child.kill(); // unblocks the reader thread
            let _ = reader.join();
            panic!("timed out waiting for {} bytes of tail output", n);
        }
    }
}

fn append(path: &PathBuf, data: &[u8]) {
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("open for append");
    f.write_all(data).expect("failed to append");
}

// --- stdin support (P0 #23) ---

#[test]
fn no_operand_reads_standard_input() {
    // GNU: echo x | tail  ->  "x"
    let out = run_tail(&[], b"x\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"x\n");
}

#[test]
fn dash_operand_reads_standard_input() {
    let out = run_tail(&["-n", "2", "-"], b"a\nb\nc\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"b\nc\n");
}

#[test]
fn bytes_option_works_on_standard_input() {
    let out = run_tail(&["-c", "3"], b"abcdef");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"def");
}

#[test]
fn follow_without_operands_reports_missing_operand() {
    // Following standard input is not supported on this build.
    let out = run_tail(&["-f"], b"");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("missing file operand"));
}

#[test]
fn follow_dash_operand_is_rejected() {
    let out = run_tail(&["-f", "-"], b"");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot follow"));
}

// --- multi-file without -f: behavior preserved ---

#[test]
fn multi_file_without_follow_keeps_headers_and_order() {
    let a = temp_path("mfa");
    let b = temp_path("mfb");
    std::fs::write(&a, b"a1\na2\n").unwrap();
    std::fs::write(&b, b"b1\nb2\n").unwrap();
    let out = run_tail(&[a.to_str().unwrap(), b.to_str().unwrap()], b"");
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
    assert_eq!(out.status.code(), Some(0));
    let expected = format!(
        "==> {} <==\na1\na2\n==> {} <==\nb1\nb2\n",
        a.display(),
        b.display()
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected);
}

// --- `-n N -f`: follow resumes at end-of-file (P0 #23) ---

#[test]
fn follow_n0_outputs_only_new_data() {
    // GNU `tail -n 0 -f`: no initial output; only appended data appears.
    // Pre-fix behavior replayed the whole file first.
    let path = temp_path("f0");
    std::fs::write(&path, b"old1\nold2\n").unwrap();
    let mut child = spawn_tail(&["-n", "0", "-f", path.to_str().unwrap()]);
    // Give a buggy build ample time to replay the old content first.
    std::thread::sleep(Duration::from_millis(600));
    append(&path, b"new1\n");
    let got = read_exact_with_deadline(&mut child, 5, Duration::from_secs(10));
    let _ = child.kill();
    let _ = std::fs::remove_file(&path);
    assert_eq!(got, b"new1\n");
}

#[test]
fn follow_n_positive_outputs_tail_then_only_new_data() {
    let path = temp_path("f5");
    std::fs::write(&path, b"l1\nl2\nl3\n").unwrap();
    let mut child = spawn_tail(&["-n", "2", "-f", path.to_str().unwrap()]);
    let initial = read_exact_with_deadline(&mut child, 6, Duration::from_secs(10));
    append(&path, b"l4\n");
    let more = read_exact_with_deadline(&mut child, 3, Duration::from_secs(10));
    let _ = child.kill();
    let _ = std::fs::remove_file(&path);
    assert_eq!(initial, b"l2\nl3\n");
    assert_eq!(more, b"l4\n");
}

#[test]
fn follow_multi_polls_every_file() {
    // Multi-file follow must not starve later operands: growth on any
    // file is emitted, with GNU-style headers on file switches.
    let a = temp_path("fa");
    let b = temp_path("fb");
    std::fs::write(&a, b"a-old\n").unwrap();
    std::fs::write(&b, b"b-old\n").unwrap();
    let mut child = spawn_tail(&[
        "-n",
        "0",
        "-f",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    std::thread::sleep(Duration::from_millis(600));
    append(&b, b"B1\n");
    // Larger than the follow poll interval so the order is deterministic.
    std::thread::sleep(Duration::from_millis(400));
    append(&a, b"A1\n");

    let expected = format!(
        "==> {} <==\n\n==> {} <==\nB1\n\n==> {} <==\nA1\n",
        a.display(),
        b.display(),
        a.display()
    );
    let got = read_exact_with_deadline(&mut child, expected.len(), Duration::from_secs(10));
    let _ = child.kill();
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
    assert_eq!(got, expected.as_bytes());
}
