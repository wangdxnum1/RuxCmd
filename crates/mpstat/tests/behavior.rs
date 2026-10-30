use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_all_cpu_row() {
    let out = cargo_bin_cmd!("mpstat").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("CPU"), "output: {text}");
    assert!(
        text.lines().any(|l| l.trim_start().starts_with("all")),
        "output: {text}"
    );
}

#[test]
fn percentage_columns_stay_in_range() {
    let out = cargo_bin_cmd!("mpstat").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    for token in text.split_whitespace() {
        if let Ok(v) = token.parse::<f64>() {
            assert!((-0.01..=100.01).contains(&v), "value {v} out of range");
        }
    }
}

#[test]
fn count_argument_adds_average_summary() {
    // mpstat DELAY COUNT: COUNT samples followed by an Average line.
    let out = cargo_bin_cmd!("mpstat").args(["2", "1"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Average:"), "output: {text}");
    assert!(text.matches("CPU").count() >= 1, "output: {text}");
}
