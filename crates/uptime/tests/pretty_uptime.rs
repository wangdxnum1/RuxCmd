use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn pretty_days_reflect_uptime_not_epoch() {
    // Regression: the Windows branch used to return seconds since the Unix
    // epoch (~20000+ days). Real uptime since boot can never reach 1000 days
    // on a GetTickCount64-based implementation.
    let output = cargo_bin_cmd!("uptime").arg("-p").output().unwrap();
    assert!(output.status.success(), "uptime -p must exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let days = stdout
        .split_whitespace()
        .find(|token| token.ends_with("天"))
        .map(|token| token.trim_end_matches('天').parse::<u64>().unwrap())
        .unwrap_or(0);
    assert!(
        days < 1000,
        "uptime -p reported {days} days (epoch bug?); output: {stdout:?}"
    );
}

#[test]
fn plain_output_is_procps_format() {
    // Round 12 B1: the default output is the procps line
    // `HH:MM:SS up N days, H:MM, 0 users` (was bare epoch seconds).
    let output = cargo_bin_cmd!("uptime").output().unwrap();
    assert!(output.status.success(), "uptime must exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains(" up ") && stdout.trim_end().ends_with(", 0 users"),
        "uptime must print the procps format, got: {stdout:?}"
    );
}
