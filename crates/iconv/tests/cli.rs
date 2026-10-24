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
