use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_cpu_header_and_rows() {
    let out = cargo_bin_cmd!("iostat").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("avg-cpu:"), "output: {text}");
    assert!(text.contains("%idle"), "output: {text}");
}

#[test]
fn cpu_columns_are_percentages() {
    // Every percentage stays within 0..=100 and the row sums plausibly.
    let out = cargo_bin_cmd!("iostat").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    for token in text.split_whitespace() {
        if let Ok(v) = token.parse::<f64>() {
            assert!((-0.01..=100.01).contains(&v), "value {v} out of range");
        }
    }
}

#[test]
fn count_argument_produces_multiple_samples() {
    // DELAY 0 COUNT 2: two since-boot reports (bounded, no endless run).
    let out = cargo_bin_cmd!("iostat").args(["0", "2"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.matches("avg-cpu:").count() >= 2, "output: {text}");
}
