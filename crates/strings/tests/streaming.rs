//! Round 13 streaming smoke tests: `strings` scans 64 KiB chunks while
//! keeping printable sequences and their absolute offsets continuous across
//! chunk boundaries (previously the whole file was read into memory).

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const CHUNK: usize = 64 * 1024;

#[test]
fn seven_bit_string_straddling_the_chunk_boundary_is_intact() {
    // 'A' x 10 at offsets CHUNK-5 .. CHUNK+4, everything else NUL.
    let mut data = vec![0u8; CHUNK * 2];
    for i in 0..10 {
        data[CHUNK - 5 + i] = b'A';
    }
    let dir = tempdir().unwrap();
    let input = dir.path().join("straddle.bin");
    fs::write(&input, &data).unwrap();

    cargo_bin_cmd!("strings")
        .args(["-t", "d"])
        .arg(&input)
        .assert()
        .success()
        .stdout(format!("{:08} AAAAAAAAAA\n", CHUNK - 5));
}

#[test]
fn utf16le_string_straddling_the_chunk_boundary_is_intact() {
    // "BBBBBBBBBB" as UTF-16LE units spanning the chunk boundary; unit at
    // CHUNK-2 starts the string, so current_string must survive the split.
    let mut data = vec![0u8; CHUNK * 2];
    let mut i = CHUNK - 2;
    while i < CHUNK - 2 + 20 {
        data[i] = b'B';
        data[i + 1] = 0x00;
        i += 2;
    }
    let dir = tempdir().unwrap();
    let input = dir.path().join("utf16.bin");
    fs::write(&input, &data).unwrap();

    cargo_bin_cmd!("strings")
        .args(["-e", "l", "-t", "d"])
        .arg(&input)
        .assert()
        .success()
        .stdout(format!("{:08} BBBBBBBBBB\n", CHUNK - 2));
}
