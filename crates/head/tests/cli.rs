//! Behavior tests for head: byte-transparent line mode, the `-n`/`-c`
//! mutual exclusion, and `-v` on standard input (Round 12 D2).
//!
//! The tests spawn the binary by its absolute build path
//! (`CARGO_BIN_EXE_head`), never a bare name, so a `head` from PATH can
//! never satisfy them; comparisons are byte-exact because line mode must be
//! binary-transparent.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run_head(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_head"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn head");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for head")
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ruxcmd-head-{}-{}.txt", tag, std::process::id()))
}

// --- P1: byte-transparent line mode ------------------------------------------

/// `printf 'a\r\nb' | head -2` must emit the raw bytes `a\r\nb`: the `\r` of
/// the CRLF terminator is preserved and the unterminated last line gets no
/// newline appended (pre-fix: `lines()` + `writeln!` rewrote both).
#[test]
fn line_mode_keeps_crlf_and_unterminated_last_line() {
    let out = run_head(&["-n", "2"], b"a\r\nb");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\r\nb");
}

#[test]
fn line_mode_keeps_lf_lines_intact() {
    let out = run_head(&["-n", "2"], b"a\nb\nc\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\nb\n");
}

/// Lines that are not valid UTF-8 pass through unchanged; the pre-fix
/// `lines()` aborted with "cannot read line" on them.
#[test]
fn line_mode_passes_non_utf8_bytes_through() {
    let out = run_head(&["-n", "1"], b"\xFF\xFE bin\nrest\n");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    assert_eq!(out.stdout, b"\xFF\xFE bin\n");
}

/// NUL bytes must survive line mode (binary safety).
#[test]
fn line_mode_passes_nul_bytes_through() {
    let out = run_head(&["-n", "1"], b"a\x00b\nc\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\x00b\n");
}

// --- P1: `-n` and `-c` are mutually exclusive (usage error, exit 1) ----------

#[test]
fn n_and_c_together_fail_with_exit_1() {
    let out = run_head(&["-n", "1", "-c", "5"], b"data\n");
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot specify both"), "stderr: {}", stderr);
}

#[test]
fn c_alone_still_works() {
    let out = run_head(&["-c", "5"], b"data\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"data\n");
}

// --- P1: `-v` prints the header for standard input ---------------------------

/// GNU names standard input `standard input` in headers (same as tail).
#[test]
fn v_prints_stdin_header() {
    let out = run_head(&["-v"], b"x\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"==> standard input <==\nx\n");
}

#[test]
fn v_prints_file_header_for_single_file() {
    let path = temp_path("v");
    std::fs::write(&path, b"a\n").unwrap();
    let out = run_head(&["-v", path.to_str().unwrap()], b"");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(0));
    let expected = format!("==> {} <==\na\n", path.display());
    assert_eq!(out.stdout, expected.as_bytes());
}
