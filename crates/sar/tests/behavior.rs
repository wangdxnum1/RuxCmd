use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_timestamped_cpu_report() {
    let out = cargo_bin_cmd!("sar").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("CPU"), "output: {text}");
    assert!(text.contains("all"), "output: {text}");
    // A timestamp line (HH:MM:SS) is present.
    assert!(
        text.lines().any(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            cols.first()
                .map(|t| t.matches(':').count() == 2)
                .unwrap_or(false)
        }),
        "output: {text}"
    );
}

#[test]
fn cpu_row_percentages_stay_in_range() {
    // Only the CPU report rows (marked by the `all` column) are percentage
    // based; the memory report prints megabyte values.
    let out = cargo_bin_cmd!("sar").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if !line.split_whitespace().any(|c| c == "all") {
            continue;
        }
        for token in line.split_whitespace().skip(2) {
            if let Ok(v) = token.parse::<f64>() {
                assert!((-0.01..=100.01).contains(&v), "value {v} in line: {line}");
            }
        }
    }
}

#[test]
fn count_flag_is_bounded() {
    let out = cargo_bin_cmd!("sar").args(["1", "2"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.matches("all").count() >= 2, "output: {text}");
}
