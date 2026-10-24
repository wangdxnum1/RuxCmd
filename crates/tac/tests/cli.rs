//! Behavior tests for tac record semantics.
//!
//! Expected outputs verified against GNU coreutils 8.32 (`tac`, `tac -b`,
//! `tac -s`): each record carries its separator (after the record body by
//! default, before it with `-b`); a trailing separator does not create an
//! extra empty record; a final segment without separator is emitted without
//! one.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_tac(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tac"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn tac");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for tac")
}

#[test]
fn trailing_separator_produces_no_empty_record() {
    // GNU: printf 'a\nb\n' | tac  ->  "b\na\n"
    let out = run_tac(&[], b"a\nb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"b\na\n");
}

#[test]
fn last_record_without_separator_stays_separatorless() {
    // GNU: printf 'a\nb' | tac  ->  "b" + "a\n" == "ba\n"
    let out = run_tac(&[], b"a\nb");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"ba\n");
}

#[test]
fn empty_input_produces_empty_output() {
    let out = run_tac(&[], b"");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

#[test]
fn single_separator_is_the_only_record() {
    // GNU: printf '\n' | tac  ->  "\n"
    let out = run_tac(&[], b"\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\n");
}

#[test]
fn custom_separator_travels_with_records() {
    // GNU: printf 'a::b::' | tac -s '::'  ->  "b::a::"
    let out = run_tac(&["-s", "::"], b"a::b::");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"b::a::");
}

#[test]
fn custom_separator_no_trailing_segment() {
    // GNU: printf 'a::b' | tac -s '::'  ->  "b" + "a::" == "ba::"
    let out = run_tac(&["-s", "::"], b"a::b");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"ba::");
}

#[test]
fn before_mode_matches_gnu() {
    // GNU: printf 'a\nb\n' | tac -b  ->  "\n\nba"
    // (records: "a", "\nb", "\n"; the trailing separator forms a
    // separator-plus-empty-content record)
    let out = run_tac(&["-b"], b"a\nb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\n\nba");

    // GNU: printf 'a\nb' | tac -b  ->  "\nba"
    let out = run_tac(&["-b"], b"a\nb");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\nba");
}

#[test]
fn file_input_reversed_per_record() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("ruxcmd-tac-test-{}.txt", std::process::id()));
    std::fs::write(&path, b"1\n2\n3\n").expect("write temp file");
    let out = Command::new(env!("CARGO_BIN_EXE_tac"))
        .arg(&path)
        .output()
        .expect("run tac on file");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"3\n2\n1\n");
}
