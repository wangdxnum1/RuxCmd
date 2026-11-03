use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::process::Output;
use tempfile::TempDir;

fn run(args: &[&str], input: &[u8]) -> Output {
    cargo_bin_cmd!("xxd")
        .args(args)
        .write_stdin(input.to_vec())
        .output()
        .unwrap()
}

fn stdout(args: &[&str], input: &[u8]) -> String {
    String::from_utf8(run(args, input).stdout).unwrap()
}

fn run_file(args: &[String], path: &str) -> Output {
    cargo_bin_cmd!("xxd").args(args).arg(path).output().unwrap()
}

fn sanitize(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[test]
fn normal_columns_format() {
    assert_eq!(
        stdout(&["-c", "4"], &[0, 1, 2, 3]),
        "00000000: 0001 0203  ....\n"
    );
}

#[test]
fn group_sizes() {
    assert_eq!(
        stdout(&["-g", "1", "-c", "4"], &[0, 1, 2, 3]),
        "00000000: 00 01 02 03  ....\n"
    );
    assert_eq!(
        stdout(&["-g", "4", "-c", "8"], &[0, 1, 2, 3, 4, 5, 6, 7]),
        "00000000: 00010203 04050607  ........\n"
    );
    assert_eq!(
        stdout(&["-g", "0", "-c", "8"], &[0, 1, 2, 3, 4, 5, 6, 7]),
        "00000000: 0001020304050607  ........\n"
    );
}

#[test]
fn length_truncates() {
    assert_eq!(
        stdout(&["-l", "2", "-c", "4"], &[0, 1, 2, 3]),
        "00000000: 0001       ..\n"
    );
    assert_eq!(stdout(&["-l", "0"], &[0, 1]), "");
    assert_eq!(
        stdout(&["-l", "0x2", "-p", "-c", "4"], &[0xab, 0xcd, 0xef, 0x01]),
        "abcd\n"
    );
}

#[test]
fn seek_positive_and_relative() {
    let data = [0, 1, 2, 3, 4, 5];
    assert_eq!(
        stdout(&["-s", "2", "-c", "4"], &data),
        "00000002: 0203 0405  ....\n"
    );
    assert_eq!(
        stdout(&["-s", "+2", "-c", "4"], &data),
        "00000002: 0203 0405  ....\n"
    );
}

#[test]
fn seek_from_end_on_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("six.bin");
    fs::write(&path, [0u8, 1, 2, 3, 4, 5]).unwrap();
    let args = vec![
        "-s".to_string(),
        "-3".to_string(),
        "-c".to_string(),
        "4".to_string(),
    ];
    let out = run_file(&args, &path.to_string_lossy());
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "00000003: 0304 05    ...\n"
    );
}

#[test]
fn seek_past_end_fails_with_exit_four() {
    let out = run(&["-s", "-100"], &[0, 1, 2, 3]);
    assert_eq!(out.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot seek"));
}

#[test]
fn uppercase_normal_and_plain() {
    assert_eq!(
        stdout(&["-u", "-c", "4"], &[0xab, 0xcd, 0xef, 0x01]),
        "00000000: ABCD EF01  ....\n"
    );
    assert_eq!(
        stdout(&["-p", "-u", "-c", "4"], &[0xab, 0xcd, 0xef, 0x01]),
        "ABCDEF01\n"
    );
}

#[test]
fn plain_hex_default_width_and_columns() {
    let data: Vec<u8> = (0..64u8).collect();
    let out = stdout(&["-p"], &data);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0].len(), 60, "default plain width is 30 bytes");
    assert_eq!(out, stdout(&["-p"], &data));
    assert_eq!(
        stdout(&["-p", "-c", "4"], &[0xab, 0xcd, 0xef, 0x01]),
        "abcdef01\n"
    );
}

#[test]
fn include_from_stdin_has_no_declaration() {
    // GNU suppresses the name and length lines when reading stdin.
    assert_eq!(stdout(&["-i"], &[0, 1, 2, 3]), "  0x00, 0x01, 0x02, 0x03\n");
    assert_eq!(
        stdout(&["-i", "-c", "2"], &[0, 1, 2]),
        "  0x00, 0x01,\n  0x02\n"
    );
    assert_eq!(stdout(&["-i", "-u"], &[0, 1]), "  0X00, 0X01\n");
    assert_eq!(stdout(&["-i", "-b"], &[0, 1]), "  0b00000000, 0b00000001\n");
}

#[test]
fn include_file_uses_sanitized_name_and_length() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("a-b.c.bin");
    fs::write(&path, [1u8, 2]).unwrap();
    let text = path.to_string_lossy().into_owned();
    let name = sanitize(&text);
    let out = run_file(&["-i".to_string()], &text);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!(
            "unsigned char {}[] = {{\n  0x01, 0x02\n}};\nunsigned int {}_len = 2;\n",
            name, name
        )
    );
}

#[test]
fn include_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.bin");
    fs::write(&path, []).unwrap();
    let text = path.to_string_lossy().into_owned();
    let name = sanitize(&text);
    let out = run_file(&["-i".to_string()], &text);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!(
            "unsigned char {}[] = {{\n}};\nunsigned int {}_len = 0;\n",
            name, name
        )
    );
}

#[test]
fn little_endian_reverses_inside_groups() {
    assert_eq!(
        stdout(&["-e", "-g", "2", "-c", "4"], &[0, 1, 2, 3]),
        "00000000: 0100 0302  ....\n"
    );
    assert_eq!(
        stdout(&["-e", "-c", "4"], &[0, 1, 2, 3, 4]),
        "00000000: 03020100  ....\n00000004:       04  .\n"
    );
}

#[test]
fn binary_dump_format() {
    assert_eq!(
        stdout(&["-b", "-c", "2"], &[0, 1]),
        "00000000: 00000000 00000001  ..\n"
    );
    assert_eq!(
        stdout(&["-b", "-u", "-c", "2"], &[0, 1]),
        "00000000: 00000000 00000001  ..\n"
    );
}

#[test]
fn columns_zero_keeps_the_default() {
    let data: Vec<u8> = (0..20u8).collect();
    assert_eq!(stdout(&["-c", "0"], &data), stdout(&[], &data));
}

#[test]
fn reject_conflicting_style_options() {
    for args in [["-b", "-p"], ["-e", "-p"], ["-e", "-b"]] {
        let out = run(&args, &[0, 1]);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("only one of"),
            "{args:?}"
        );
    }
}

#[test]
fn revert_applies_a_non_negative_offset() {
    let out = run(&["-r", "-s", "4"], b"00000000: 4142\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, vec![0, 0, 0, 0, 0x41, 0x42]);
}

#[test]
fn help_lists_supported_options() {
    let out = run(&["--help"], b"");
    let text = String::from_utf8_lossy(&out.stdout);
    for option in [
        "--len",
        "--seek",
        "--group",
        "--plain",
        "--include",
        "--little-endian",
        "--uppercase",
        "--binary",
    ] {
        assert!(text.contains(option), "help is missing {option}");
    }
}
