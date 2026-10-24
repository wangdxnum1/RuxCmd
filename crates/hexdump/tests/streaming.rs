//! Round 13 streaming smoke tests: `hexdump` reads the input 16 bytes at a
//! time (after seeking past `-s`), so a 2 MiB file is never held in memory.
//! Equivalence is verified structurally: line/offset layout checks plus a
//! full round-trip that rebuilds the input from the hex columns of `-C`.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

const TWO_MIB: usize = 2 * 1024 * 1024;

#[test]
fn canonical_two_megabyte_input_round_trips() {
    // Patterned (non-trivial) 2 MiB: 131072 rows of 16 bytes.
    let mut data = Vec::with_capacity(TWO_MIB);
    let mut x: u8 = 0;
    while data.len() < TWO_MIB {
        data.push(x);
        x = x.wrapping_add(7);
    }
    let dir = tempdir().unwrap();
    let input = dir.path().join("big.bin");
    fs::write(&input, &data).unwrap();

    let output = cargo_bin_cmd!("hexdump")
        .arg("-C")
        .arg(&input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    // 131072 data rows plus the final offset line 00200000 (2 MiB in hex).
    assert_eq!(lines.len(), TWO_MIB / 16 + 1);
    assert!(text.ends_with("\n00200000\n"));
    for (i, line) in lines[..TWO_MIB / 16].iter().enumerate() {
        assert!(
            line.starts_with(&format!("{:08x}  ", i * 16)),
            "row {i}: {line}"
        );
    }

    // Round-trip: parse the 16 hex pairs of every row and rebuild the input.
    // Pair i starts at column 10+3i for i<8 and 35+3*(i-8) for i>=8 (the
    // extra space sits at column 34).
    let mut rebuilt = Vec::with_capacity(TWO_MIB);
    for line in &lines[..TWO_MIB / 16] {
        assert_eq!(&line.as_bytes()[59..61], b" |");
        assert!(line.ends_with('|'));
        for i in 0..16 {
            let start = if i < 8 { 10 + 3 * i } else { 35 + 3 * (i - 8) };
            let pair = &line[start..start + 2];
            rebuilt.push(u8::from_str_radix(pair, 16).unwrap());
        }
    }
    assert_eq!(rebuilt, data);
}

#[test]
fn skip_and_length_slice_a_small_file() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bytes.bin");
    fs::write(&input, (0u8..32).collect::<Vec<u8>>()).unwrap();

    // -s 5 -n 4: one short row starting at 0x05, then the final offset.
    let expected = format!(
        "00000005  05 06 07 08 {}{} |....|\n00000009\n",
        "   ".repeat(4),
        " ".repeat(1 + 3 * 8),
    );
    cargo_bin_cmd!("hexdump")
        .args(["-C", "-s", "5", "-n", "4"])
        .arg(&input)
        .assert()
        .success()
        .stdout(expected);

    // -n 0: nothing at all, not even the trailing offset.
    cargo_bin_cmd!("hexdump")
        .args(["-C", "-n", "0"])
        .arg(&input)
        .assert()
        .success()
        .stdout("");
}

#[test]
fn skip_at_or_past_end_of_file_prints_nothing() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("four.bin");
    fs::write(&input, [0x41u8, 0x42, 0x43, 0x44]).unwrap();
    for skip in ["4", "400"] {
        cargo_bin_cmd!("hexdump")
            .args(["-C", "-s", skip])
            .arg(&input)
            .assert()
            .success()
            .stdout("");
    }
}

#[test]
fn skip_on_stdin_discards_bytes() {
    let expected = format!(
        "00000001  42 {}{} |B|\n00000002\n",
        "   ".repeat(7),
        " ".repeat(1 + 3 * 8),
    );
    cargo_bin_cmd!("hexdump")
        .args(["-C", "-s", "1", "-n", "1"])
        .write_stdin("AB")
        .assert()
        .success()
        .stdout(expected);
}
