use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn input(dir: &TempDir, name: &str, bytes: &[u8]) -> String {
    let p = dir.path().join(name);
    fs::write(&p, bytes).unwrap();
    p.to_string_lossy().into_owned()
}

fn run(args: &[&str], stdin_bytes: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("expand")
        .args(args)
        .write_stdin(stdin_bytes)
        .output()
        .unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn default_tab_stops_every_eight_columns() {
    let (out, err, code) = run(&[], b"a\tb\n");
    assert_eq!(code, Some(0));
    assert!(err.is_empty());
    assert_eq!(out, b"a       b\n");
}

#[test]
fn crlf_is_preserved_verbatim() {
    let (out, _err, code) = run(&[], b"a\r\nb\tc\r\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a\r\nb       c\r\n");
}

#[test]
fn unterminated_last_line_stays_unterminated() {
    let (out, _err, code) = run(&[], b"a\tb");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a       b");
}

#[test]
fn periodic_tab_option() {
    let (out, _err, code) = run(&["-t", "3"], b"a\tb\tc\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a  b  c\n");
}

#[test]
fn explicit_tab_list() {
    let (out, _err, code) = run(&["-t", "3,5"], b"a\tb\tc\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a  b c\n");
}

#[test]
fn tab_after_last_list_stop_becomes_one_space() {
    let (out, _err, code) = run(&["-t", "3,5"], b"abcde\tf\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"abcde f\n");
}

#[test]
fn empty_list_items_are_skipped() {
    let (out, _err, code) = run(&["-t", "3,,5"], b"a\tb\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a  b\n");
}

#[test]
fn initial_only_expands_leading_blank_run() {
    let (out, _err, code) = run(&["-i", "-t", "2"], b"\ta\tb\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"  a\tb\n");
}

#[test]
fn zero_tab_size_is_rejected_like_gnu() {
    let (_out, err, code) = run(&["-t", "0"], b"");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("expand: tab size cannot be 0"),
        "stderr: {}",
        err
    );
}

#[test]
fn non_ascending_tab_list_is_rejected_like_gnu() {
    let (_out, err, code) = run(&["-t", "5,3"], b"");
    assert_eq!(code, Some(1));
    assert!(
        err.contains("expand: tab sizes must be ascending"),
        "stderr: {}",
        err
    );
}

#[test]
fn invalid_utf8_bytes_pass_through_with_column_accounting() {
    // 0xFF counts as one column; the following tab still expands.
    let (out, _err, code) = run(&["-t", "4"], &[0xFF, b'\t', b'x', b'\n']);
    assert_eq!(code, Some(0));
    assert_eq!(out, vec![0xFF, b' ', b' ', b' ', b'x', b'\n']);
}

#[test]
fn multibyte_characters_count_as_one_column() {
    let (out, _err, code) = run(&["-t", "4"], "中\tb\n".as_bytes());
    assert_eq!(code, Some(0));
    assert_eq!(out, "中   b\n".as_bytes());
}

#[test]
fn file_operand_and_dash_stdin() {
    let dir = TempDir::new().unwrap();
    let f = input(&dir, "in.txt", b"x\ty\n");
    let (out, _err, code) = run(&[&f], b"");
    assert_eq!(code, Some(0));
    assert_eq!(out, b"x       y\n");
}

#[test]
fn missing_file_reports_error_and_exits_one() {
    let (_out, err, code) = run(&["definitely-missing-file.txt"], b"");
    assert_eq!(code, Some(1));
    assert!(err.contains("expand:"), "stderr: {}", err);
}
