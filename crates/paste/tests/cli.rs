//! Behavior tests for paste: CRLF terminator stripping and `-d` character
//! list cycling by output column (Round 12 F1).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_paste"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn paste");
    child
        .stdin
        .take()
        .expect("stdin pipe")
        .write_all(b"")
        .expect("failed to close stdin");
    child.wait_with_output().expect("failed to wait for paste")
}

fn temp_file(tag: &str, contents: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ruxcmd-paste-{}-{}.txt",
        tag,
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("failed to write fixture");
    path
}

// --- P1: CRLF terminators must not leave a stray \r in the output ----------

/// `paste f1 f2` on CRLF files must emit `a\t1\nb\t2\n`; pre-fix
/// `trim_end_matches('\n')` kept the `\r` of `\r\n` in every field.
#[test]
fn crlf_terminators_are_stripped() {
    let f1 = temp_file("crlf1", "a\r\nb\r\n");
    let f2 = temp_file("crlf2", "1\r\n2\r\n");

    let out = run(&[
        f1.to_string_lossy().as_ref(),
        f2.to_string_lossy().as_ref(),
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\t1\nb\t2\n");
    assert_eq!(out.stderr, b"");
}

// --- P2 (fixed here too): -d character list cycles by output column --------

/// `paste -d ',;' f1 f2 f3` must use `,` between columns 1-2 and `;` between
/// columns 2-3; pre-fix the whole string was used as one separator.
#[test]
fn delimiter_list_cycles_by_column() {
    let f1 = temp_file("cycle1", "a\nb\n");
    let f2 = temp_file("cycle2", "1\n2\n");
    let f3 = temp_file("cycle3", "x\ny\n");

    let out = run(&[
        "-d",
        ",;",
        f1.to_string_lossy().as_ref(),
        f2.to_string_lossy().as_ref(),
        f3.to_string_lossy().as_ref(),
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a,1;x\nb,2;y\n");
}

/// Serial mode cycles through the list per joined line position.
#[test]
fn serial_mode_cycles_delimiter_list() {
    let f1 = temp_file("serial", "a\nb\nc\n");

    let out = run(&["-s", "-d", ",;", f1.to_string_lossy().as_ref()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a,b;c\n");
}
