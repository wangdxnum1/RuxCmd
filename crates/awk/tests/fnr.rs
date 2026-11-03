//! FNR semantics, verified against gawk 5.4.0 (MSYS2 / Git for Windows):
//! FNR resets for every input file while NR keeps counting, main-input
//! `getline` advances both counters and shares the record cursor, and the
//! redirect/pipe forms leave NR and FNR alone.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn awk(dir: Option<&Path>, args: &[&str], stdin: &[u8]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_awk"));
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.as_mut().unwrap().write_all(stdin);
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn stdout(output: Output) -> String {
    assert!(output.status.success(), "awk failed: {:?}", output.status);
    String::from_utf8(output.stdout).unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("awk-fnr-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture(dir: &Path, name: &str, contents: &str) {
    std::fs::write(dir.join(name), contents).unwrap();
}

#[test]
fn fnr_resets_per_file_while_nr_continues() {
    let dir = temp_dir("reset");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "f2.txt", "d\ne\n");
    let out = awk(
        Some(&dir),
        &["{print NR, FNR, $0}", "f1.txt", "f2.txt"],
        b"",
    );
    assert_eq!(stdout(out), "1 1 a\n2 2 b\n3 3 c\n4 1 d\n5 2 e\n");
}

#[test]
fn empty_file_resets_fnr_before_the_next_file() {
    let dir = temp_dir("empty-middle");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "empty.txt", "");
    fixture(&dir, "f2.txt", "d\ne\n");
    let out = awk(
        Some(&dir),
        &["{print NR, FNR, $0}", "f1.txt", "empty.txt", "f2.txt"],
        b"",
    );
    assert_eq!(stdout(out), "1 1 a\n2 2 b\n3 3 c\n4 1 d\n5 2 e\n");
}

#[test]
fn trailing_empty_file_leaves_fnr_at_zero() {
    let dir = temp_dir("empty-tail");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "empty.txt", "");
    let out = awk(
        Some(&dir),
        &["END{print NR, FNR}", "f1.txt", "empty.txt"],
        b"",
    );
    assert_eq!(stdout(out), "3 0\n");
}

#[test]
fn getline_from_main_input_increments_nr_and_fnr() {
    let dir = temp_dir("getline-var");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "f2.txt", "d\ne\n");
    let out = awk(
        Some(&dir),
        &["{r=getline x; print NR, FNR, r, x}", "f1.txt", "f2.txt"],
        b"",
    );
    assert_eq!(stdout(out), "2 2 1 b\n4 1 1 d\n5 2 0 d\n");
}

#[test]
fn getline_from_main_input_crosses_files_and_resplits_fields() {
    let dir = temp_dir("getline-record");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "f2.txt", "d\ne\n");
    let out = awk(
        Some(&dir),
        &["{getline; print NR, FNR, NF, $0}", "f1.txt", "f2.txt"],
        b"",
    );
    assert_eq!(stdout(out), "2 2 1 b\n4 1 1 d\n5 2 1 e\n");
}

#[test]
fn getline_at_eof_keeps_the_record_and_variable() {
    let dir = temp_dir("getline-eof");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    let var_out = awk(
        Some(&dir),
        &["{x=\"keep\"; r=getline x; print NR, FNR, r, x}", "f1.txt"],
        b"",
    );
    assert_eq!(stdout(var_out), "2 2 1 b\n3 3 0 keep\n");

    let record_out = awk(
        Some(&dir),
        &["{getline; print NR, FNR, NF, $0}", "f1.txt"],
        b"",
    );
    assert_eq!(stdout(record_out), "2 2 1 b\n3 3 1 c\n");
}

#[test]
fn pipe_getline_leaves_fnr_alone() {
    let dir = temp_dir("pipe-getline");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    let out = awk(
        Some(&dir),
        &["{\"echo p q\" | getline v; print NR, FNR, v}", "f1.txt"],
        b"",
    );
    // The command is reused, so v keeps its last value once the pipe is at EOF.
    assert_eq!(stdout(out), "1 1 p q\n2 2 p q\n3 3 p q\n");
}

#[test]
fn redirect_getline_leaves_fnr_alone() {
    let dir = temp_dir("redirect-getline");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    fixture(&dir, "f2.txt", "d\ne\n");
    let out = awk(
        Some(&dir),
        &["{getline v < \"f2.txt\"; print NR, FNR, v}", "f1.txt"],
        b"",
    );
    assert_eq!(stdout(out), "1 1 d\n2 2 e\n3 3 e\n");
}

#[test]
fn stdin_counts_fnr_and_nr_together() {
    let out = awk(None, &["{print NR, FNR, $0}"], b"x\ny\n");
    assert_eq!(stdout(out), "1 1 x\n2 2 y\n");
}

#[test]
fn fnr_assignment_carries_into_the_next_record() {
    let dir = temp_dir("fnr-assign");
    fixture(&dir, "f1.txt", "a\nb\nc\n");
    let out = awk(
        Some(&dir),
        &[
            "{printf \"%d/%d \", NR, FNR; if (FNR==1) FNR=100} END{print \"\"; print \"end\", NR, FNR}",
            "f1.txt",
        ],
        b"",
    );
    assert_eq!(stdout(out), "1/1 2/101 3/102 \nend 3 102\n");
}
