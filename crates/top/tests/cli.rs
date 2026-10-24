use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn header_has_real_uptime_and_cpu_percentages() {
    let out = cargo_bin_cmd!("top")
        .args(["-i", "2", "-d", "1"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let header = stdout
        .lines()
        .find(|l| l.starts_with("top - "))
        .expect("header line");
    assert!(header.contains(" up "), "header: {}", header);
    assert!(!header.contains("load average"), "header: {}", header);

    let cpu = stdout
        .lines()
        .find(|l| l.starts_with("%Cpu(s):"))
        .expect("cpu line");
    // us + id should cover most of the scale (single-digit fraction tolerance).
    let nums: Vec<f64> = cpu
        .split(',')
        .filter_map(|part| {
            part.trim()
                .split_whitespace()
                .find_map(|v| v.parse::<f64>().ok())
        })
        .collect();
    // Column order: us, sy, ni, id, ... — label-free numeric extraction.
    let us = nums.first().copied().unwrap_or(0.0);
    let id = nums.get(3).copied().unwrap_or(0.0);
    assert!(us + id > 50.0, "cpu line: {} (nums {:?})", cpu, nums);
}
