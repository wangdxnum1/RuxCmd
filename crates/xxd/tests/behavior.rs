use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

fn run(args: &[&str], input: &[u8]) -> Output {
    cargo_bin_cmd!("xxd")
        .args(args)
        .write_stdin(input.to_vec())
        .output()
        .unwrap()
}

fn dump(input: &[u8]) -> Vec<u8> {
    run(&[], input).stdout
}

fn revert(input: &[u8]) -> Vec<u8> {
    run(&["-r"], input).stdout
}

#[test]
fn hex_dump_default_format() {
    let out = cargo_bin_cmd!("xxd")
        .write_stdin(b"hello\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with("00000000:"), "output: {text}");
    assert!(text.contains("6865 6c6c"), "output: {text}");
}

#[test]
fn empty_input_produces_no_output() {
    let out = cargo_bin_cmd!("xxd").write_stdin(b"").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

#[test]
fn revert_round_trips() {
    let out = cargo_bin_cmd!("xxd")
        .write_stdin(b"hello")
        .output()
        .unwrap();
    let hex_dump = String::from_utf8_lossy(&out.stdout).into_owned();

    let out = cargo_bin_cmd!("xxd")
        .args(["-r"])
        .write_stdin(hex_dump.as_bytes())
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"hello");
}

#[test]
fn revert_ignores_ascii_column_with_hex_like_text() {
    // "0123" renders as 3031 3233 with 0123 in the ASCII column; only the
    // hex body is data.
    let dumped = dump(b"0123");
    assert_eq!(revert(&dumped), b"0123");
}

#[test]
fn revert_binary_bytes_round_trip() {
    let data = [0x00u8, 0xff, 0x41, 0x42, 0x2e, 0x7f, 0x0a, 0x0d];
    assert_eq!(revert(&dump(&data)), data);
}

#[test]
fn revert_single_byte_round_trip() {
    for byte in [0x00u8, 0xff, 0x41] {
        let data = [byte];
        assert_eq!(revert(&dump(&data)), data, "byte {byte:#04x}");
    }
}

#[test]
fn revert_multi_line_round_trip() {
    let data: Vec<u8> = (0..40u8).collect();
    assert_eq!(revert(&dump(&data)), data);
}

#[test]
fn revert_honours_column_width() {
    for cols in ["3", "8"] {
        let data: Vec<u8> = (0..10u8).collect();
        let dumped = run(&["-c", cols], &data).stdout;
        assert_eq!(revert(&dumped), data, "cols {cols}");
    }
}

#[test]
fn revert_expands_star_folded_zero_lines() {
    let data = vec![0u8; 64];
    let dumped = dump(&data);
    let text = String::from_utf8(dumped).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    // Keep the first and last address lines, fold the middle ones.
    let folded = format!("{}\n*\n{}\n", lines[0], lines[lines.len() - 1]);
    assert_eq!(revert(folded.as_bytes()), data);
}

#[test]
fn revert_zero_fills_address_gaps() {
    let fixture = b"00000000: 4142\n00000010: 4344\n";
    let mut expected = vec![0u8; 0x12];
    expected[0] = 0x41;
    expected[1] = 0x42;
    expected[0x10] = 0x43;
    expected[0x11] = 0x44;
    assert_eq!(revert(fixture), expected);
}

#[test]
fn revert_ignores_blank_and_address_only_lines() {
    let fixture = b"\n00000000:\n00000010: 4142\n";
    assert_eq!(revert(fixture), {
        let mut expected = vec![0u8; 0x12];
        expected[0x10] = 0x41;
        expected[0x11] = 0x42;
        expected
    });
}

#[test]
fn revert_accepts_dash_group_separators_and_odd_nibbles() {
    assert_eq!(revert(b"00000000: 41-42\n"), b"AB");
    // GNU xxd drops the dangling nibble instead of padding it.
    assert_eq!(revert(b"00000000: 414\n"), b"A");
}

#[test]
fn revert_accepts_crlf_dump_lines() {
    let dumped = dump(b"line one\nline two\n");
    let text = String::from_utf8(dumped).unwrap().replace('\n', "\r\n");
    assert_eq!(revert(text.as_bytes()), b"line one\nline two\n");
}
