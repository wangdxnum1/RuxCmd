use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::io::Write as _;
use std::process::Stdio;
use std::time::Duration;
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

/// Feed stdin one byte at a time with pauses so the child's reads observe
/// short reads, the only way a pipe differs from a regular file. Write
/// errors are tolerated: dd may exit early (e.g. count reached) and close
/// the pipe.
fn run_slow_pipe(args: &[&str], dir: &TempDir, chunks: &[u8]) -> (Vec<u8>, String, Option<i32>) {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dd"))
        .args(args)
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for byte in chunks {
        if stdin.write_all(&[*byte]).is_err() {
            break;
        }
        if stdin.flush().is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(60));
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

// --- conv=sync (GNU dd 8.32: pad partial input blocks with NULs up to ibs;
//     records in are counted on the raw read, before padding) ---

#[test]
fn conv_sync_pads_short_read_with_nuls() {
    let dir = TempDir::new().unwrap();
    let (out, err, code) = run(&["of=out.bin", "bs=8", "conv=sync"], &dir, b"abc");
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert!(err.contains("1+0 records out"), "stderr: {err}");
    assert!(err.contains("8 bytes copied"), "stderr: {err}");
    assert_eq!(
        fs::read(dir.path().join("out.bin")).unwrap(),
        b"abc\0\0\0\0\0"
    );
}

#[test]
fn conv_sync_counts_records_before_padding() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&["of=out.bin", "bs=8", "conv=sync"], &dir, b"ABCDEFGHIJK");
    assert_eq!(code, Some(0));
    assert!(err.contains("1+1 records in"), "stderr: {err}");
    assert!(err.contains("2+0 records out"), "stderr: {err}");
    assert!(err.contains("16 bytes copied"), "stderr: {err}");
    let content = fs::read(dir.path().join("out.bin")).unwrap();
    assert_eq!(
        content,
        b"ABCDEFGH"
            .iter()
            .chain(b"IJK\0\0\0\0\0")
            .copied()
            .collect::<Vec<u8>>()
    );
}

#[test]
fn conv_sync_notrunc_pads_prefix_keeps_tail() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("out.bin"), b"0123456789ABCDEF").unwrap();
    let (_out, err, code) = run(&["of=out.bin", "bs=8", "conv=sync,notrunc"], &dir, b"abc");
    assert_eq!(code, Some(0));
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert!(err.contains("1+0 records out"), "stderr: {err}");
    let content = fs::read(dir.path().join("out.bin")).unwrap();
    assert_eq!(content.len(), 16);
    assert_eq!(&content[..8], b"abc\0\0\0\0\0");
    assert_eq!(&content[8..], b"89ABCDEF");
}

// --- fullblock (GNU dd 8.32 has iflag=fullblock; conv=fullblock is our
//     alias) accumulate short pipe reads into full ibs-sized blocks ---

#[test]
fn conv_fullblock_accumulates_short_reads() {
    let dir = TempDir::new().unwrap();
    let (out, err, code) = run_slow_pipe(
        &["of=out.bin", "bs=8", "count=1", "conv=fullblock"],
        &dir,
        b"abcdefgh",
    );
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.contains("1+0 records in"), "stderr: {err}");
    assert!(err.contains("1+0 records out"), "stderr: {err}");
    assert!(err.contains("8 bytes copied"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"abcdefgh");
}

#[test]
fn iflag_fullblock_accumulates_short_reads() {
    let dir = TempDir::new().unwrap();
    let (out, err, code) = run_slow_pipe(
        &["of=out.bin", "bs=8", "count=1", "iflag=fullblock"],
        &dir,
        b"abcdefgh",
    );
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.contains("1+0 records in"), "stderr: {err}");
    assert!(err.contains("1+0 records out"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"abcdefgh");
}

#[test]
fn pipe_without_fullblock_stops_at_short_read() {
    let dir = TempDir::new().unwrap();
    let (out, err, code) = run_slow_pipe(&["of=out.bin", "bs=8", "count=1"], &dir, b"abcdefgh");
    assert_eq!(code, Some(0));
    assert!(out.is_empty());
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"a");
}

#[test]
fn iflag_fullblock_with_sync_pads_eof_partial() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run_slow_pipe(
        &["of=out.bin", "bs=8", "iflag=fullblock", "conv=sync"],
        &dir,
        b"abcde",
    );
    assert_eq!(code, Some(0));
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert!(err.contains("1+0 records out"), "stderr: {err}");
    assert_eq!(
        fs::read(dir.path().join("out.bin")).unwrap(),
        b"abcde\0\0\0"
    );
}

#[test]
fn invalid_iflag_fails() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&["of=out.bin", "iflag=bogus"], &dir, b"");
    assert_eq!(code, Some(1));
    assert!(err.contains("invalid input flag"), "stderr: {err}");
}

// --- conv=nocreat (GNU dd 8.32: the output file must already exist; dd
//     fails instead of creating it) ---

#[test]
fn conv_nocreat_fails_when_output_missing() {
    let dir = TempDir::new().unwrap();
    let (_out, err, code) = run(&["of=out.bin", "bs=4", "conv=nocreat"], &dir, b"abc");
    assert_eq!(code, Some(1));
    assert!(err.contains("failed to open 'out.bin'"), "stderr: {err}");
    assert!(!dir.path().join("out.bin").exists());
}

#[test]
fn conv_nocreat_opens_existing_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("out.bin"), vec![b'Z'; 16]).unwrap();
    let (_out, _err, code) = run(&["of=out.bin", "bs=4", "conv=nocreat"], &dir, b"abc");
    assert_eq!(code, Some(0));
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"abc");
}

#[test]
fn conv_nocreat_notrunc_overwrites_prefix() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("out.bin"), b"abcdefgh").unwrap();
    let (_out, err, code) = run(&["of=out.bin", "bs=8", "conv=nocreat,notrunc"], &dir, b"XY");
    assert_eq!(code, Some(0));
    assert!(err.contains("0+1 records in"), "stderr: {err}");
    assert_eq!(fs::read(dir.path().join("out.bin")).unwrap(), b"XYcdefgh");
}
