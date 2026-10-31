use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn run(args: &[&str], dir: &TempDir, stdin: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("dd")
        .args(args)
        .current_dir(dir.path())
        .write_stdin(stdin)
        .output()
        .unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn bs_count_copy_reports_gnu_records() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"ABCDEFGHIJKLMNOP").unwrap();
    let (out, err, code) = run(&["if=src.bin", "of=out.bin", "bs=4", "count=2"], &dir, b"");
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.contains("2+0 records in"), "stderr: {err}");
    assert!(err.contains("2+0 records out"), "stderr: {err}");
    assert!(err.contains("8 bytes copied"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"ABCDEFGH");
}

#[test]
fn skip_reads_from_offset() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"ABCDEFGHIJKLMNOP").unwrap();
    let (_out, err, code) = run(&["if=src.bin", "of=out.bin", "bs=4", "skip=2"], &dir, b"");
    assert_eq!(code, Some(0));
    assert!(err.contains("2+0 records in"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"IJKLMNOP");
}

#[test]
fn seek_with_notrunc_zero_fills_and_keeps_tail() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"ABCDEFGHIJKLMNOP").unwrap();
    let (_out, _err, code) = run(
        &[
            "if=src.bin",
            "of=out.bin",
            "bs=16",
            "seek=1",
            "conv=notrunc",
        ],
        &dir,
        b"",
    );
    assert_eq!(code, Some(0));
    let content = fs::read(dir.path().join("out.bin")).unwrap();
    assert_eq!(content.len(), 32);
    assert_eq!(&content[..16], &[0u8; 16]);
    assert_eq!(&content[16..], b"ABCDEFGHIJKLMNOP");
}

#[test]
fn default_output_truncates() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("out.bin"), vec![b'Z'; 64]).unwrap();
    let (_out, _err, code) = run(&["of=out.bin", "bs=8", "count=1"], &dir, b"12345678");
    assert_eq!(code, Some(0));
    let content = fs::read(dir.path().join("out.bin")).unwrap();
    assert_eq!(content, b"12345678");
}

#[test]
fn partial_trailing_block_counts_as_partial() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"abc").unwrap();
    let (_out, err, code) = run(&["if=src.bin", "of=out.bin", "bs=4"], &dir, b"");
    assert_eq!(code, Some(0));
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert!(err.contains("0+1 records out"), "stderr: {err}");
}

#[test]
fn status_none_is_silent() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"data").unwrap();
    let (_out, err, code) = run(
        &["if=src.bin", "of=out.bin", "bs=4", "status=none"],
        &dir,
        b"",
    );
    assert_eq!(code, Some(0));
    assert!(err.is_empty(), "stderr: {err}");
}

#[test]
fn size_suffixes_apply() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&["of=out.bin", "bs=1K", "count=2"], &dir, &vec![b'x'; 2048]);
    assert_eq!(code, Some(0));
    assert!(err.contains("2048 bytes copied"), "stderr: {err}");
}

#[test]
fn unknown_operand_fails() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&["bogus=1"], &dir, b"");
    assert_eq!(code, Some(1));
    assert!(err.contains("unrecognized operand"), "stderr: {err}");
}

#[test]
fn unknown_conversion_fails() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("src.bin"), b"x").unwrap();
    let (_out, err, code) = run(&["if=src.bin", "of=out.bin", "conv=bogus"], &dir, b"");
    assert_eq!(code, Some(1));
    assert!(err.contains("invalid conversion"), "stderr: {err}");
}
