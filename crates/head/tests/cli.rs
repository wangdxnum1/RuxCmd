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

#[test]
fn nul_mode_preserves_embedded_lf_cr_and_unterminated_records() {
    for flag in ["-z", "--zero-terminated"] {
        let out = run_head(&[flag, "-n", "2"], b"a\nx\0b\r\0c");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(out.stdout, b"a\nx\0b\r\0");
        let out = run_head(&[flag, "-n", "9"], b"a\0b");
        assert_eq!(out.stdout, b"a\0b");
        assert!(run_head(&[flag], b"").stdout.is_empty());
        let out = run_head(&[flag, "-c", "4"], b"a\nx\0b\0");
        assert_eq!(out.stdout, b"a\nx\0");
    }
}

#[test]
fn nul_multi_file_headers_still_use_lf_separators() {
    let first = temp_path("nul-head-first");
    let second = temp_path("nul-head-second");
    std::fs::write(&first, b"a\0").unwrap();
    std::fs::write(&second, b"b\0").unwrap();
    let out = run_head(
        &[
            "-z",
            "-n",
            "1",
            first.to_str().unwrap(),
            second.to_str().unwrap(),
        ],
        b"",
    );
    let _ = std::fs::remove_file(&first);
    let _ = std::fs::remove_file(&second);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        out.stdout,
        format!(
            "==> {} <==\na\0\n==> {} <==\nb\0",
            first.display(),
            second.display()
        )
        .as_bytes()
    );
}

// --- 0.4.6: negative counts (`-n -K` / `-c -K` = all but the last K) --------
//
// GNU coreutils 8.32, measured locally:
// - `head -n -3` on 12 lines prints lines 1..9 (all but the last 3);
// - `head -c -5` on 39 bytes prints 34 bytes;
// - `head -n -0` / `-c -0` print EVERYTHING (the sign is part of the number
//   string, so `-0` is not `0`);
// - a negative K larger than the input prints nothing, exit 0;
// - invalid numbers fail with `head: invalid number of lines: 'abc'`, exit 1.

const TWELVE_LINES: &[u8] = b"l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\nl9\nl10\nl11\nl12\n";

#[test]
fn negative_lines_on_stdin_print_all_but_last_k() {
    let out = run_head(&["-n", "-3"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, &TWELVE_LINES[..9 * 3]);
}

#[test]
fn negative_bytes_on_stdin_print_all_but_last_k() {
    let out = run_head(&["-c", "-5"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, &TWELVE_LINES[..39 - 5]);
}

/// `-n -5` and `-n 5` must not collapse to the same behavior.
#[test]
fn negative_and_positive_line_counts_differ() {
    let negative = run_head(&["-n", "-3"], TWELVE_LINES).stdout;
    let positive = run_head(&["-n", "3"], TWELVE_LINES).stdout;
    assert_eq!(positive, b"l1\nl2\nl3\n");
    assert_eq!(negative, &TWELVE_LINES[..9 * 3]);
    assert_ne!(negative, positive);
}

/// `-n -0` means "all but the last 0 records" = everything, unlike `-n 0`.
#[test]
fn negative_zero_counts_print_everything() {
    let out = run_head(&["-n", "-0"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, TWELVE_LINES);
    let out = run_head(&["-c", "-0"], TWELVE_LINES);
    assert_eq!(out.stdout, TWELVE_LINES);
    let out = run_head(&["-n", "0"], TWELVE_LINES);
    assert_eq!(out.stdout, b"");
}

#[test]
fn negative_lines_on_file_print_all_but_last_k() {
    let path = temp_path("neg-lines");
    std::fs::write(&path, TWELVE_LINES).unwrap();
    let out = run_head(&["-n", "-3", path.to_str().unwrap()], b"");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, &TWELVE_LINES[..9 * 3]);
}

/// GNU headers and blank separators still apply per file with negative
/// counts (`head -n -2 a.txt b.txt` measured: `==> a <==\nA\n\n==> b <==\nB\n`).
#[test]
fn negative_lines_multi_file_keep_headers() {
    let first = temp_path("neg-head-first");
    let second = temp_path("neg-head-second");
    std::fs::write(&first, b"A\nB\nC\n").unwrap();
    std::fs::write(&second, b"B\nC\nD\n").unwrap();
    let out = run_head(
        &[
            "-n",
            "-2",
            first.to_str().unwrap(),
            second.to_str().unwrap(),
        ],
        b"",
    );
    let _ = std::fs::remove_file(&first);
    let _ = std::fs::remove_file(&second);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        out.stdout,
        format!(
            "==> {} <==\nA\n\n==> {} <==\nB\n",
            first.display(),
            second.display()
        )
        .as_bytes()
    );
}

/// A negative K larger than the input prints nothing, exit 0 (GNU measured).
#[test]
fn negative_count_larger_than_input_prints_nothing() {
    let out = run_head(&["-n", "-5"], b"a\nb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"");
    let out = run_head(&["-c", "-100"], b"tiny");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"");
}

#[test]
fn negative_lines_in_zero_terminated_mode() {
    let out = run_head(&["-z", "-n", "-2"], b"a\0b\0c\0d\0e\0");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a\0b\0c\0");
}

#[test]
fn verbose_header_still_printed_with_negative_count() {
    let path = temp_path("neg-verbose");
    std::fs::write(&path, b"A\nB\nC\n").unwrap();
    let out = run_head(&["-v", "-n", "-2", path.to_str().unwrap()], b"");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        out.stdout,
        format!("==> {} <==\nA\n", path.display()).as_bytes()
    );
}

/// The `-n`/`-c` mutual exclusion also holds when the counts are negative.
#[test]
fn n_and_c_together_fail_even_when_negative() {
    let out = run_head(&["-n", "-3", "-c", "-5"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot specify both"), "stderr: {}", stderr);
}

/// Invalid numbers must follow the GNU wording and exit 1, matching
/// `head: invalid number of lines: 'abc'`.
#[test]
fn invalid_numbers_fail_with_gnu_message_and_exit_1() {
    let out = run_head(&["-n", "abc"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid number of lines: 'abc'"),
        "stderr: {stderr}"
    );
    let out = run_head(&["-c", "1x"], TWELVE_LINES);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid number of bytes: '1x'"),
        "stderr: {stderr}"
    );
}
