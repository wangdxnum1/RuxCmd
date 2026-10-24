use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn two_frames_keep_seventeen_columns() {
    let out = cargo_bin_cmd!("vmstat").args(["1", "2"]).assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let data_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('r'))
        .collect();
    assert!(data_lines.len() >= 2, "stdout: {:?}", stdout);
    for l in data_lines {
        assert_eq!(l.split_whitespace().count(), 17, "line: {:?}", l);
    }
}
