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
    assert_cpu_percentages(cpu);
}

fn assert_cpu_percentages(cpu: &str) {
    let nums: Vec<f64> = cpu
        .split(',')
        .filter_map(|part| {
            part.trim()
                .split_whitespace()
                .find_map(|v| v.parse::<f64>().ok())
        })
        .collect();
    // User, system, and idle together cover the scale, regardless of host load.
    assert_eq!(nums.len(), 8, "cpu line: {}", cpu);
    assert!(
        nums.iter().all(|n| (0.0..=100.0).contains(n)),
        "cpu line: {} (nums {:?})",
        cpu,
        nums
    );
    // Each nonzero column is rounded to one decimal place.
    let total: f64 = nums.iter().sum();
    assert!((total - 100.0).abs() <= 0.2, "cpu line: {}", cpu);
}

#[test]
fn cpu_percentages_accept_dominant_system_time() {
    assert_cpu_percentages(
        "%Cpu(s): 6.7 us, 53.3 sy, 0.0 ni, 40.0 id, 0.0 wa, 0.0 hi, 0.0 si, 0.0 st",
    );
}
