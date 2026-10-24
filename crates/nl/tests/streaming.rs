//! Round 13 streaming smoke test: `nl` must number a 2 MiB input without
//! ever holding the input (or a fully materialized output string) in memory.
//! Equivalence with the previous buffered implementation is verified by
//! building the expected output exactly (the format is trivial) and by
//! round-tripping the content through the emitted prefixes.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

/// 32768 lines x 64 bytes (63 content + '\n') = exactly 2 MiB.
const LINES: usize = 32 * 1024;
const BODY_LEN: usize = 63;

fn line_body(i: usize) -> String {
    format!("data line {:05}{}", i, "x".repeat(48))
}

#[test]
fn two_megabyte_input_is_numbered_correctly() {
    let mut input = String::with_capacity(LINES * (BODY_LEN + 1));
    for i in 0..LINES {
        input.push_str(&line_body(i));
        input.push('\n');
    }
    assert_eq!(input.len(), 2 * 1024 * 1024);

    let dir = tempdir().unwrap();
    let file = dir.path().join("big.txt");
    fs::write(&file, &input).unwrap();

    let output = cargo_bin_cmd!("nl")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();

    // Fully computed expectation: "number(6) + tab + content + \n" per line.
    let mut expected = String::with_capacity(LINES * (BODY_LEN + 8));
    for i in 0..LINES {
        expected.push_str(&format!("{:>6}\t{}\n", i + 1, line_body(i)));
    }
    assert_eq!(output, expected);

    // Structural spot checks (line count, first/last prefixes, round-trip).
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), LINES);
    assert_eq!(&lines[0][..7], "     1\t");
    assert_eq!(&lines[LINES - 1][..7], format!("{:>6}\t", LINES));
    for idx in [0, 1, LINES / 2, LINES - 1] {
        let line = lines[idx];
        // Strip "number + tab" prefix; the rest must be the original content.
        let content = &line[line.find('\t').unwrap() + 1..];
        assert_eq!(content, line_body(idx));
    }
}

/// The same input through a pipe exercises the stdin path of the streaming
/// reader (no file BufReader involved).
#[test]
fn two_megabyte_stdin_stream_matches() {
    let mut input = String::with_capacity(LINES * (BODY_LEN + 1));
    for i in 0..LINES {
        input.push_str(&line_body(i));
        input.push('\n');
    }

    let output = cargo_bin_cmd!("nl")
        .write_stdin(input.as_bytes())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();

    assert_eq!(output.len(), LINES * (7 + BODY_LEN + 1));
    // First and last numbered lines.
    assert!(output.starts_with("     1\tdata line 00000"));
    assert!(output.ends_with(&format!("{:>6}\t{}\n", LINES, line_body(LINES - 1))));
}
