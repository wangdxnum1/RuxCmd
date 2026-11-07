//! Integration tests for the avg-cpu section (real GetSystemTimes
//! percentages) and the `-d` device report backed by IOCTL_DISK_PERFORMANCE.

use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Output};

/// Always spawn via the absolute path cargo provides: `cargo test` prepends
/// target/debug to PATH, so a bare "iostat" could resolve to an unrelated
/// binary dropped there.
fn iostat(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iostat"))
        .args(args)
        .output()
        .expect("spawn iostat")
}

/// Extract the numeric value line of every avg-cpu report:
/// [user, nice, system, iowait, steal, idle].
fn cpu_rows(stdout: &str) -> Vec<Vec<f64>> {
    let mut rows = Vec::new();
    let mut lines = stdout.lines();
    while let Some(line) = lines.next() {
        if line.starts_with("avg-cpu:") {
            let values = lines
                .next()
                .unwrap_or_else(|| panic!("no value line after {line:?}"));
            let cols: Vec<f64> = values
                .split_whitespace()
                .map(|t| t.parse().unwrap_or_else(|e| panic!("column {t:?}: {e}")))
                .collect();
            assert_eq!(cols.len(), 6, "six avg-cpu columns in {values:?}");
            rows.push(cols);
        }
    }
    rows
}

fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "exit {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `iostat 1 2` exits by itself after two frames whose percentages add up.
#[test]
fn two_frames_exit_with_real_percentages() {
    let out = iostat(&["1", "2"]);
    assert_success(&out);
    let rows = cpu_rows(&String::from_utf8_lossy(&out.stdout));
    assert_eq!(rows.len(), 2, "exactly two reports");
    for (i, row) in rows.iter().enumerate() {
        let sum = row[0] + row[2] + row[5]; // user + system + idle
        assert!(
            (98.0..=102.0).contains(&sum),
            "frame {i}: usr+sys+idle = {sum}"
        );
    }
}

/// Plain `iostat` prints a single since-boot report and exits.
#[test]
fn plain_invocation_prints_one_report() {
    let out = iostat(&[]);
    assert_success(&out);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("%idle"), "sysstat avg-cpu header kept");
    assert_eq!(cpu_rows(&text).len(), 1);
}

/// The device report lists the real physical drives.
#[test]
fn device_report_lists_physical_drives() {
    // -d prints the sysstat device report; the drives come from
    // IOCTL_DISK_PERFORMANCE and keep the PhysicalDriveN names.
    let out = cargo_bin_cmd!("iostat").arg("-d").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Device"), "header missing: {text}");
    assert!(text.contains("PhysicalDrive0"), "drive row missing: {text}");
}
