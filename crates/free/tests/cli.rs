use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn total_flag_adds_mem_plus_swap_row() {
    let out = cargo_bin_cmd!("free").arg("-t").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("Mem:"), "stdout: {:?}", stdout);
    assert!(stdout.contains("Swap:"), "stdout: {:?}", stdout);
    assert!(
        stdout.contains("Total:"),
        "-t must add a Total row: {:?}",
        stdout
    );
}

#[test]
fn human_units_use_procps_suffixes() {
    let out = cargo_bin_cmd!("free").arg("-h").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(
        stdout.contains("Gi"),
        "-h must print Gi suffixes: {:?}",
        stdout
    );
}

#[test]
fn seconds_with_count_repeats_frames() {
    let out = cargo_bin_cmd!("free")
        .args(["-s", "1", "-c", "2"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let frames = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with("Mem:"))
        .count();
    assert_eq!(frames, 2, "expected 2 frames: {:?}", stdout);
}
