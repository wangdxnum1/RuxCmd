//! Byte-transparency tests: tee must forward stdin bytes unchanged to
//! stdout and every file operand — CRLF, NUL, high bytes, invalid UTF-8,
//! and a missing final newline all pass through (P0 #24, GNU tee is byte
//! transparent).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ruxcmd-tee-{}-{}.bin", tag, std::process::id()))
}

fn run_tee(args: &[&str], stdin_data: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tee"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn tee");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_data)
        .expect("failed to write stdin");
    child.wait_with_output().expect("failed to wait for tee")
}

fn sample_bytes() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"crlf line\r\n");
    data.extend_from_slice(b"bare-cr\r");
    data.extend_from_slice(b"lf line\n");
    data.push(0x00);
    data.push(0x01);
    data.push(0x80); // invalid UTF-8 on its own
    data.extend_from_slice(&[0xC3, 0xA9]); // "é" in UTF-8
    data.extend_from_slice(&[0xFE, 0xFF]);
    data.extend_from_slice(b"no final newline");
    data
}

#[test]
fn binary_bytes_reach_stdout_and_all_files() {
    let data = sample_bytes();
    let f1 = temp_path("b1");
    let f2 = temp_path("b2");
    let out = run_tee(&[f1.to_str().unwrap(), f2.to_str().unwrap()], &data);
    let c1 = std::fs::read(&f1).expect("read first file");
    let c2 = std::fs::read(&f2).expect("read second file");
    let _ = std::fs::remove_file(&f1);
    let _ = std::fs::remove_file(&f2);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, data);
    assert_eq!(c1, data);
    assert_eq!(c2, data);
}

#[test]
fn crlf_and_missing_final_newline_pass_through_unchanged() {
    let data = b"a\r\nb\r\ntail-no-newline".as_slice();
    let f = temp_path("crlf");
    let out = run_tee(&[f.to_str().unwrap()], data);
    let content = std::fs::read(&f).expect("read output file");
    let _ = std::fs::remove_file(&f);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, data);
    assert_eq!(content, data);
}

#[test]
fn no_files_copies_stdin_to_stdout() {
    let out = run_tee(&[], b"\x00plain\r\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"\x00plain\r\n");
}

#[test]
fn append_mode_preserves_bytes() {
    let f = temp_path("app");
    std::fs::write(&f, b"keep\r\n").expect("seed file");
    let data = &b"added\x00\x80\n"[..];
    let out = run_tee(&["-a", f.to_str().unwrap()], data);
    let content = std::fs::read(&f).expect("read appended file");
    let _ = std::fs::remove_file(&f);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, data);
    assert_eq!(content, b"keep\r\nadded\x00\x80\n");
}

#[test]
fn multi_chunk_input_stays_byte_exact() {
    // Cross the 8 KB pump-chunk boundary several times.
    let mut data = vec![0u8; 3 * 8192 + 11];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 251) as u8; // includes NUL, CR, LF, and high bytes
    }
    let f = temp_path("big");
    let out = run_tee(&[f.to_str().unwrap()], &data);
    let content = std::fs::read(&f).expect("read output file");
    let _ = std::fs::remove_file(&f);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, data);
    assert_eq!(content, data);
}
