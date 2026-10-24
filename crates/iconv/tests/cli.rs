use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

#[test]
fn converts_gbk_to_utf8() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("gbk.txt");
    // GBK bytes for "中文"
    fs::write(&input, b"\xd6\xd0\xce\xc4").unwrap();
    cargo_bin_cmd!("iconv")
        .args(["-f", "gbk", "-t", "utf-8"])
        .arg(&input)
        .assert()
        .success()
        .stdout("中文");
}

#[test]
fn from_gbk_decodes_utf8_bom_bytes_strictly_as_gbk() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bom_gbk.bin");
    // UTF-8 BOM (EF BB BF) followed by GBK bytes; with strict -f handling the
    // BOM bytes are decoded as GBK characters, not sniffed as UTF-8.
    fs::write(&input, b"\xef\xbb\xbf\xd6\xd0\xce").unwrap();
    cargo_bin_cmd!("iconv")
        .args(["-f", "gbk", "-t", "utf-8"])
        .arg(&input)
        .assert()
        .success()
        .stdout("锘恐形");
}

#[test]
fn from_utf8_keeps_utf8_bom_in_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bom.txt");
    fs::write(&input, b"\xef\xbb\xbfhi").unwrap();
    cargo_bin_cmd!("iconv")
        .args(["-f", "utf-8", "-t", "utf-8"])
        .arg(&input)
        .assert()
        .success()
        .stdout("\u{feff}hi");
}

#[test]
fn list_covers_more_than_the_old_28_entries() {
    let output = cargo_bin_cmd!("iconv")
        .arg("--list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    for name in ["GBK", "ISO-8859-16", "KOI8-R", "windows-1252", "GB18030"] {
        assert!(text.contains(name), "missing {name} in --list output");
    }
}

// --- Round 13 streaming: 8 KiB chunk boundaries must split characters safely ---

#[test]
fn gbk_sequence_straddling_the_8k_chunk_boundary_decodes() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("boundary.bin");
    // 8191 ASCII bytes then GBK "中文" + "中": the first byte of 中 (D6) is
    // the last byte of the first 8 KiB chunk, D0 arrives in the next one.
    let mut data = vec![b'a'; 8191];
    data.extend_from_slice(b"\xd6\xd0\xce\xc4");
    data.extend_from_slice(b"\xd6\xd0"); // a final pair entirely in chunk 2
    fs::write(&input, &data).unwrap();
    cargo_bin_cmd!("iconv")
        .args(["-f", "gbk", "-t", "utf-8"])
        .arg(&input)
        .assert()
        .success()
        .stdout(format!("{}中文中", "a".repeat(8191)));
}

#[test]
fn utf8_to_gbk_encode_straddles_the_8k_chunk_boundary() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("boundary-utf8.txt");
    // 8191 ASCII chars then UTF-8 "中文" (6 bytes): the 3-byte sequence for
    // 中 is decoded in one chunk and must encode to GBK D6 D0 on the way out.
    let mut data = vec![b'a'; 8191];
    data.extend_from_slice("中文中文".as_bytes());
    fs::write(&input, &data).unwrap();
    let output = cargo_bin_cmd!("iconv")
        .args(["-f", "utf-8", "-t", "gbk"])
        .arg(&input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut expected = vec![b'a'; 8191];
    expected.extend_from_slice(b"\xd6\xd0\xce\xc4\xd6\xd0\xce\xc4");
    assert_eq!(output, expected);
}

#[test]
fn truncated_gbk_tail_becomes_replacement_in_stream() {
    // A dangling GBK lead byte at EOF must decode to U+FFFD exactly like
    // the previous whole-buffer decode with last = true.
    let output = cargo_bin_cmd!("iconv")
        .args(["-f", "gbk", "-t", "utf-8"])
        .write_stdin(b"ab\xd6".as_slice())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(output, "ab\u{FFFD}".as_bytes());
}
