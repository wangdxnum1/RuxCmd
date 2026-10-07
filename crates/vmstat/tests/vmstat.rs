//! Regression tests for P0 #46: vmstat sampled memory once before the frame
//! loop, so every line of `vmstat 1 5` carried identical swpd/free values.

use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Parse vmstat output into data rows (rows whose first column is numeric).
/// Asserts every line carries exactly the 17 vmstat columns.
fn parse_frames(stdout: &[u8]) -> Vec<Vec<u64>> {
    let text = String::from_utf8_lossy(stdout);
    let mut frames = Vec::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(cols.len(), 17, "expected 17 columns, got line {line:?}");
        if cols[0].parse::<u64>().is_ok() {
            frames.push(
                cols.iter()
                    .map(|c| c.parse::<u64>().expect("numeric column"))
                    .collect(),
            );
        }
    }
    frames
}

/// Column layout: r b swpd free buff cache si so bi bo in cs us sy id wa st.
const FREE_COLUMN: usize = 3;

#[test]
fn two_frames_have_two_data_rows_of_17_columns() {
    let output = cargo_bin_cmd!("vmstat")
        .args(["1", "2"])
        .output()
        .expect("run vmstat");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frames = parse_frames(&output.stdout);
    assert_eq!(frames.len(), 2, "expected exactly two data rows");
}

/// `vmstat 2 2` must sample memory once per frame. While vmstat sleeps
/// between frame 1 and frame 2, this test commits and touches 512 MiB, so the
/// free column of frame 2 must be at least 128 MiB below frame 1. Before the
/// fix both rows showed the same sample taken before the loop.
#[test]
fn memory_is_resampled_between_frames() {
    let child = Command::new(env!("CARGO_BIN_EXE_vmstat"))
        .args(["2", "2"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn vmstat");

    // Frame 1 is sampled immediately at startup; commit the ballast afterwards
    // so it can only influence frame 2.
    thread::sleep(Duration::from_millis(300));
    const BYTES: usize = 512 * 1024 * 1024;
    let mut ballast: Vec<u8> = Vec::new();
    let committed = ballast.try_reserve_exact(BYTES).is_ok();
    if committed {
        // Non-zero fill forces the pages to be committed, not lazily mapped.
        ballast.resize(BYTES, 0xAA);
    }

    let output = child.wait_with_output().expect("wait for vmstat");
    drop(ballast);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frames = parse_frames(&output.stdout);
    assert_eq!(frames.len(), 2);
    let (first, second) = (frames[0][FREE_COLUMN], frames[1][FREE_COLUMN]);
    // 512 MiB committed => free (reported in KiB) must drop by >= 128 MiB.
    // The large margin absorbs unrelated system-wide memory fluctuations.
    if committed {
        assert!(
            second + 128 * 1024 <= first,
            "free column was not resampled per frame: frame1={first} KiB frame2={second} KiB"
        );
    }
}
