use assert_cmd::cargo::cargo_bin_cmd;
use std::io::Write;

#[test]
fn counts_stdin_lines() {
    let output = cargo_bin_cmd!("wc")
        .arg("-l")
        .write_stdin("one\ntwo\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(output).unwrap().trim(), "2 <stdin>");
}

#[test]
fn chars_counts_utf8_characters_not_bytes() {
    // "你好\n" is 3 characters but 7 bytes.
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all("你好\n".as_bytes()).unwrap();
    f.flush().unwrap();

    let output = cargo_bin_cmd!("wc")
        .arg("-m")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("3 {}", f.path().display())
    );

    let output = cargo_bin_cmd!("wc")
        .arg("-c")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("7 {}", f.path().display())
    );
}

#[test]
fn chars_counts_across_buffer_boundary() {
    // 3000x "中" (3 bytes each) + "\n" = 9001 bytes; the 8 KiB read chunk
    // splits a multi-byte sequence, so the counter must carry it over.
    let content: String = "中".repeat(3000) + "\n";
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();

    let output = cargo_bin_cmd!("wc")
        .arg("-m")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("3001 {}", f.path().display())
    );
}

#[test]
fn invalid_utf8_falls_back_to_byte_count() {
    // GNU wc -m errors on invalid multibyte input; this build counts bytes
    // instead (declared deviation). 0xFF is invalid UTF-8.
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(&[0xFF, 0xAB, b'\n']).unwrap();
    f.flush().unwrap();

    let output = cargo_bin_cmd!("wc")
        .arg("-m")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("3 {}", f.path().display())
    );
}

#[test]
fn max_line_length_excludes_newline() {
    // GNU: printf 'ab\n' | wc -L  ->  2
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"ab\n").unwrap();
    f.flush().unwrap();

    let output = cargo_bin_cmd!("wc")
        .arg("-L")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("2 {}", f.path().display())
    );
}

#[test]
fn max_line_length_counts_final_line_without_newline() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"ab").unwrap();
    f.flush().unwrap();

    let output = cargo_bin_cmd!("wc")
        .arg("-L")
        .arg(f.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output).unwrap().trim(),
        format!("2 {}", f.path().display())
    );
}

#[test]
fn missing_file_reports_error_and_exits_one() {
    cargo_bin_cmd!("wc")
        .arg("no-such-file-ruxcmd-wc-test.txt")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("wc:"));
}
