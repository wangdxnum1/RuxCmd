use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn run_stdin(bytes: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("rev").write_stdin(bytes).output().unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn reverses_ascii_lines() {
    let (out, err, code) = run_stdin(b"abc\ndef\n");
    assert_eq!(code, Some(0));
    assert!(err.is_empty());
    assert_eq!(out, b"cba\nfed\n");
}

#[test]
fn reverses_multibyte_characters_not_bytes() {
    // util-linux rev is multibyte aware: characters reverse, not bytes.
    let (out, _err, code) = run_stdin("中文\n".as_bytes());
    assert_eq!(code, Some(0));
    assert_eq!(out, "文中\n".as_bytes());
}

#[test]
fn empty_lines_stay_empty() {
    let (out, _err, code) = run_stdin(b"\n\nx\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"\n\nx\n");
}

#[test]
fn unterminated_last_line_stays_unterminated() {
    let (out, _err, code) = run_stdin(b"abc");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"cba");
}

#[test]
fn crlf_newline_moves_carriage_return_with_content() {
    // Byte reversal puts \r at the front of the reversed content; the
    // line terminator stays \n.
    let (out, _err, code) = run_stdin(b"ab\r\ncd\r\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"\rba\n\rdc\n");
}

#[test]
fn invalid_utf8_reverses_as_raw_bytes() {
    // Byte-transparent fallback: the invalid sequence is reversed byte by
    // byte, not rejected and not re-encoded.
    let (out, _err, code) = run_stdin(&[0x61, 0xFF, 0x62, b'\n']);
    assert_eq!(code, Some(0));
    assert_eq!(out, vec![0x62, 0xFF, 0x61, b'\n']);
}

#[test]
fn file_operand_reverses_and_dash_reads_stdin() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("in.txt");
    fs::write(&f, b"file line\n").unwrap();
    let path = f.to_string_lossy().into_owned();
    let out = cargo_bin_cmd!("rev").arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"enil elif\n");

    let out = cargo_bin_cmd!("rev")
        .arg("-")
        .write_stdin(b"pipe\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"epip\n");
}

#[test]
fn missing_file_reports_error_and_exits_one() {
    cargo_bin_cmd!("rev")
        .arg("definitely-missing-file.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("rev:"));
}
