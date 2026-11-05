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
        .filter_map(|part| part.split_whitespace().find_map(|v| v.parse::<f64>().ok()))
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

#[test]
fn version_flag_reports_the_version() {
    let out = cargo_bin_cmd!("top").args(["--version"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "top 0.1.0");
}

#[test]
fn single_frame_without_delay_prints_expected_blocks() {
    // -d 0 keeps the run immediate, so the test never sleeps on the sampling
    // interval.
    let out = cargo_bin_cmd!("top")
        .args(["-i", "1", "-d", "0"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert_eq!(
        stdout.matches("top - ").count(),
        1,
        "exactly one frame: {}",
        stdout
    );
    for marker in ["Tasks:", "%Cpu(s):", "KiB Mem :", "KiB Swap:", "PID"] {
        assert!(stdout.contains(marker), "missing {marker}: {}", stdout);
    }
}

#[test]
fn process_table_is_capped_at_twenty_rows() {
    let out = cargo_bin_cmd!("top")
        .args(["-i", "1", "-d", "0"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let header = lines
        .iter()
        .position(|line| line.starts_with("PID "))
        .expect("process header");
    let mut rows = 0;
    for line in lines.iter().skip(header + 1) {
        if line.starts_with('=') {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let pid = line.split_whitespace().next().expect("pid column");
        assert!(
            pid.parse::<u32>().is_ok(),
            "process row should start with a PID: {line}"
        );
        rows += 1;
    }
    assert!((1..=20).contains(&rows), "process rows: {}", rows);
}
