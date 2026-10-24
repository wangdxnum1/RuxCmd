use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn shows_help() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn shows_version() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ps"));
}

#[test]
fn runs_default() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"));
}

#[test]
fn custom_columns() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-o", "pid,ppid,comm"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"))
        .stdout(predicate::str::contains("PPID"))
        .stdout(predicate::str::contains("COMM"));
}

#[test]
fn no_headers() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["--no-headers", "-o", "pid,comm"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID").not());
}

#[test]
fn forest_renders() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["--forest", "-e", "-o", "pid,comm"]);
    cmd.assert().success();
}

#[test]
fn unknown_column_warns() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-o", "pid,nonsense"]);
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("unknown column"));
}

#[test]
fn list_columns_works() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--list-columns");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pid"))
        .stdout(predicate::str::contains("comm"));
}

#[test]
fn extra_full_and_long_stderr_is_clean() {
    // Regression: -F used to request the unregistered columns `sz`/`psr` and
    // -l requested `wchan`, so every run printed "ps: warning: unknown
    // column ..." to stderr and silently dropped those columns.
    for flag in ["-F", "-l"] {
        let mut cmd = Command::cargo_bin("ps").unwrap();
        cmd.arg(flag);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("PID"))
            .stderr(predicate::str::is_empty());
    }
}

#[test]
fn pmem_values_are_bounded_by_total_memory() {
    // Regression for the hardcoded 8 GB %MEM denominator: with real total
    // physical memory the sum of all %MEM values cannot exceed 100% because
    // the sum of all working sets cannot exceed total physical memory. With
    // the old 8 GB denominator this summed to well over 100% on machines
    // with more than 8 GB of RAM and busy working sets.
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-o", "pid,pmem"]);
    let out = cmd.assert().success();
    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    let mut lines = text.lines();
    let header = lines.next().expect("ps must print a header row");
    assert!(
        header.contains("%MEM"),
        "expected a %MEM header, got: {header}"
    );

    let mut sum = 0.0f64;
    let mut rows = 0usize;
    for line in lines {
        assert!(!line.trim().is_empty(), "unexpected empty row: {line:?}");
        let pmem = line
            .split_whitespace()
            .nth(1)
            .unwrap_or_else(|| panic!("missing %MEM value in row: {line:?}"))
            .parse::<f64>()
            .unwrap_or_else(|e| panic!("cannot parse %MEM value in row {line:?}: {e}"));
        assert!(
            (0.0..=100.0).contains(&pmem),
            "%MEM must be within 0..=100, got {pmem} in row: {line:?}"
        );
        sum += pmem;
        rows += 1;
    }
    assert!(rows > 0, "ps must list at least one process");
    // At least one real process must hold a non-zero share of memory.
    assert!(sum > 0.0, "no process reports a non-zero %MEM");
    // One-decimal rounding may inflate the sum by up to 0.05 per row.
    let slack = rows as f64 * 0.05 + 1e-9;
    assert!(
        sum <= 100.0 + slack,
        "sum of %MEM values ({sum}) exceeds total physical memory (100%)"
    );
}

#[test]
fn pcpu_shows_numbers_for_queryable_processes() {
    // Round 12 C2: %CPU was permanently "?" because the prev-sample pipeline
    // was never wired. It is now a lifetime average and numeric for any
    // process whose times are readable.
    let output = Command::cargo_bin("ps")
        .unwrap()
        .args(["-o", "pid,pcpu", "--sort=-pcpu"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let numeric: Vec<&str> = stdout
        .lines()
        .skip(1)
        .filter(|l| {
            let cpu = l.split_whitespace().nth(1).unwrap_or("?");
            cpu.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .collect();
    assert!(
        numeric.len() >= 5,
        "expected several numeric %CPU rows: {:?}",
        stdout
    );
}

#[test]
fn stat_column_reports_sleeping_not_running() {
    let output = Command::cargo_bin("ps")
        .unwrap()
        .args(["-o", "pid,stat"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stat_values: Vec<&str> = stdout
        .lines()
        .skip(1)
        .filter_map(|l| l.split_whitespace().nth(1))
        .collect();
    assert!(
        stat_values.iter().any(|s| s.contains('S')),
        "expected at least one S row: {:?}",
        stdout
    );
}

#[test]
fn headers_repeat_is_rejected() {
    // Round 12 C2: the never-implemented --headers-repeat was removed.
    Command::cargo_bin("ps")
        .unwrap()
        .arg("--headers-repeat=2")
        .assert()
        .failure();
}
