use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn own_pid_column_shows_numeric_pids() {
    let out = cargo_bin_cmd!("netstat").arg("-o").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("PID"), "header: {:?}", stdout);
    let pid_rows: Vec<&str> = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with("TCP") || l.trim_start().starts_with("UDP"))
        .collect();
    assert!(!pid_rows.is_empty(), "no rows: {:?}", stdout);
    assert!(
        pid_rows.iter().any(|l| l
            .split_whitespace()
            .last()
            .unwrap_or("")
            .parse::<u32>()
            .is_ok()),
        "no numeric PID: {:?}",
        pid_rows
    );
}

#[test]
fn listening_hidden_without_all_flag() {
    let plain = String::from_utf8_lossy(
        &cargo_bin_cmd!("netstat")
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .to_string();
    let all = String::from_utf8_lossy(
        &cargo_bin_cmd!("netstat")
            .arg("-a")
            .assert()
            .success()
            .get_output()
            .stdout,
    )
    .to_string();
    assert!(all.contains("LISTENING"), "-a must show LISTENING");
    assert!(
        !plain.lines().any(|l| l.contains("LISTENING")),
        "default should hide LISTENING: {:?}",
        plain
    );
}
