//! Integration tests for the Round 12 / batch A2 P1 remediation: CPU rows
//! carry real GetSystemTimes percentages behind a HH:MM:SS local timestamp,
//! memory stays real with a checked GlobalMemoryStatusEx call, finite runs
//! end with Average rows, and the unimplemented `-d` flag is rejected.

use std::process::{Command, Output};

/// Always spawn via the absolute path cargo provides: `cargo test` prepends
/// target/debug to PATH, so a bare "sar" could resolve to an unrelated binary
/// dropped there.
fn sar(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sar"))
        .args(args)
        .output()
        .expect("spawn sar")
}

fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "exit {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// True for strings shaped like HH:MM:SS.
fn is_hms(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 8
        && b[2] == b':'
        && b[5] == b':'
        && b.iter()
            .enumerate()
            .all(|(i, &c)| i == 2 || i == 5 || c.is_ascii_digit())
}

/// Rows tagged with the given marker ("all" for CPU, "Mem" for memory).
/// Returns (first column, remaining numeric columns).
fn tagged_rows<'a>(stdout: &'a str, marker: &str) -> Vec<(&'a str, Vec<f64>)> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let first = cols.next()?;
            let tag = cols.next()?;
            if tag != marker {
                return None;
            }
            let nums: Vec<f64> = cols
                .map(|t| t.parse().unwrap_or_else(|e| panic!("column {t:?}: {e}")))
                .collect();
            Some((first, nums))
        })
        .collect()
}

/// `sar -u 1 2` exits by itself after two real CPU samples plus an Average
/// row; every sample line starts with a local HH:MM:SS timestamp.
#[test]
fn cpu_rows_are_real_timestamped_and_averaged() {
    let out = sar(&["-u", "1", "2"]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = tagged_rows(&text, "all");
    assert_eq!(rows.len(), 3, "two samples plus Average");
    let samples: Vec<&(&str, Vec<f64>)> = rows.iter().filter(|(l, _)| l != &"Average:").collect();
    assert_eq!(samples.len(), 2);
    for (label, row) in samples {
        assert!(
            is_hms(label),
            "sample line must start with HH:MM:SS, got {label:?}"
        );
        let sum = row[0] + row[1] + row[2]; // usr + sys + idle
        assert!(
            (98.0..=102.0).contains(&sum),
            "{label}: usr+sys+idle = {sum}"
        );
    }
    let (label, avg) = rows.last().expect("Average row");
    assert_eq!(*label, "Average:");
    let sum = avg[0] + avg[1] + avg[2];
    assert!(
        (98.0..=102.0).contains(&sum),
        "Average: usr+sys+idle = {sum}"
    );
}

/// `sar -r 1 2` keeps the real memory rows with timestamps and an Average.
#[test]
fn memory_rows_stay_real_and_averaged() {
    let out = sar(&["-r", "1", "2"]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = tagged_rows(&text, "Mem");
    assert_eq!(rows.len(), 3, "two samples plus Average");
    for (label, row) in rows.iter().take(2) {
        assert!(
            is_hms(label),
            "sample line must start with HH:MM:SS, got {label:?}"
        );
        let (used, free, total) = (row[0], row[1], row[2]);
        assert!(total > 0.0, "total memory must be non-zero real data");
        assert!(
            (used + free - total).abs() <= 2.0,
            "{label}: used({used}) + free({free}) must match total({total})"
        );
    }
}

/// Plain `sar 1 2` reports both activities by default.
#[test]
fn default_reports_cpu_and_memory() {
    let out = sar(&["1", "2"]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(tagged_rows(&text, "all").len(), 3);
    assert_eq!(tagged_rows(&text, "Mem").len(), 3);
}

/// The unimplemented disk activity flag must no longer be accepted silently.
#[test]
fn disk_flag_is_rejected() {
    let out = sar(&["-d"]);
    assert!(
        !out.status.success(),
        "sar -d must be rejected, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
