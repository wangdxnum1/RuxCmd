use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

fn to_display_form(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn run(dir: &TempDir, args: &[&str]) -> (String, String, Option<i32>) {
    let out = cargo_bin_cmd!("whereis")
        .env("PATH", dir.path())
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn found_command_prints_gnu_style_line() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("tool.exe"), b"MZ").unwrap();
    let (out, _err, code) = run(&dir, &["tool"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out,
        format!("tool: {}\n", to_display_form(&dir.path().join("tool.exe")))
    );
}

#[test]
fn missing_command_prints_bare_label_and_exits_one() {
    let dir = TempDir::new().unwrap();
    let (out, _err, code) = run(&dir, &["nothing-here"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "nothing-here:\n");
}

#[test]
fn mixed_results_exit_zero() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("have.exe"), b"MZ").unwrap();
    let (out, _err, code) = run(&dir, &["have", "lack"]);
    assert_eq!(code, Some(0));
    let lines: Vec<&str> = out.trim_end().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("have: "));
    assert_eq!(lines[1], "lack:");
}

#[test]
fn manual_section_reports_nothing_on_windows() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("tool.exe"), b"MZ").unwrap();
    let (out, _err, code) = run(&dir, &["-m", "tool"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "tool:\n");
}

#[test]
fn binaries_only_flag_restricts_to_binaries() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("tool.exe"), b"MZ").unwrap();
    let (out, _err, code) = run(&dir, &["-b", "tool"]);
    assert_eq!(code, Some(0));
    assert!(out.contains("tool.exe"));
}

#[test]
fn hard_coded_personal_path_is_gone() {
    // The old implementation probed D:\develop\rust-tools; the sources must
    // not reference it anymore (behavioral guarantee via source text).
    let src = include_str!("../src/main.rs");
    assert!(!src.contains("develop\\rust-tools"));
    let _ = fs::metadata("unused");
}

#[test]
fn missing_operand_is_a_usage_error() {
    let out = cargo_bin_cmd!("whereis").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("missing operand"));
}
