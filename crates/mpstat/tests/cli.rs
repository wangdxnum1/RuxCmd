//! Integration tests for the Round 12 / batch A2 P1 remediation: the "all"
//! row carries real GetSystemTimes percentages, finite runs end with an
//! Average row, and per-CPU requests fail loudly instead of printing zeros.

use std::process::{Command, Output};

/// Always spawn via the absolute path cargo provides: `cargo test` prepends
/// target/debug to PATH, so a bare "mpstat" could resolve to an unrelated
/// binary dropped there.
fn mpstat(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mpstat"))
        .args(args)
        .output()
        .expect("spawn mpstat")
}

fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "exit {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Rows whose first column is `all` or `Average:`:
/// [usr, nice, sys, iowait, irq, soft, steal, idle].
fn data_rows(stdout: &str) -> Vec<(&str, Vec<f64>)> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut cols = line.split_whitespace();
            let first = cols.next()?;
            if first != "all" && first != "Average:" {
                return None;
            }
            let nums: Vec<f64> = cols
                .map(|t| t.parse().unwrap_or_else(|e| panic!("column {t:?}: {e}")))
                .collect();
            assert_eq!(nums.len(), 8, "eight value columns in {line:?}");
            Some((first, nums))
        })
        .collect()
}

/// `mpstat 1 2` exits by itself after two real frames plus an Average row.
#[test]
fn two_frames_exit_with_average_row() {
    let out = mpstat(&["1", "2"]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = data_rows(&text);
    assert_eq!(rows.len(), 3, "two all rows plus Average");
    assert_eq!(rows.iter().filter(|(l, _)| *l == "all").count(), 2);
    assert!(
        rows.iter().any(|(l, _)| *l == "Average:"),
        "Average row exists"
    );
    for (label, row) in &rows {
        let sum = row[0] + row[2] + row[7]; // usr + sys + idle
        assert!(
            (98.0..=102.0).contains(&sum),
            "{label}: usr+sys+idle = {sum}"
        );
    }
    // The Average row is the mean of the two frames.
    let (_, avg) = rows.last().expect("Average row");
    let frames: Vec<&Vec<f64>> = rows.iter().map(|(_, r)| r).collect();
    let mean_usr = (frames[0][0] + frames[1][0]) / 2.0;
    assert!((avg[0] - mean_usr).abs() < 0.01, "Average usr = {}", avg[0]);
}

/// Plain `mpstat` prints a single since-boot row and exits (no Average row,
/// matching sysstat's no-interval behaviour).
#[test]
fn plain_invocation_prints_one_row() {
    let out = mpstat(&[]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = data_rows(&text);
    assert_eq!(rows.len(), 1, "exactly one all row");
}

/// `-P ALL` behaves like the default single aggregate row.
#[test]
fn capital_p_all_matches_default_output() {
    let out = mpstat(&["-P", "ALL"]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    let rows = data_rows(&text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "all");
}

/// Any per-CPU request must fail loudly with exit code 1.
#[test]
fn per_cpu_output_is_rejected() {
    for args in [&["-P", "0"][..], &["-P", "3"][..], &["-P", "abc"][..]] {
        let out = mpstat(args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "args {args:?} must exit 1, stdout: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stderr).trim(),
            "mpstat: per-cpu output is not supported"
        );
    }
}
