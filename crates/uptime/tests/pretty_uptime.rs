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
fn plain_output_is_finite_seconds() {
    // The non-pretty output must stay within plausible boot uptime seconds
    // (< 1000 days), not epoch-seconds.
    let output = cargo_bin_cmd!("uptime").output().unwrap();
    assert!(output.status.success(), "uptime must exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let secs: u64 = stdout
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("uptime must print a bare number, got: {stdout:?}"));
    assert!(
        secs < 1000 * 86400,
        "uptime printed {secs} seconds (epoch bug?)"
    );
}
