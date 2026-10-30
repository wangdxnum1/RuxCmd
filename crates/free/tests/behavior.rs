use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn prints_header_and_memory_rows() {
    let out = cargo_bin_cmd!("free").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("total"), "output: {text}");
    assert!(text.contains("free"), "output: {text}");
    // At least Mem row; commonly Swap too.
    assert!(text.contains("Mem"), "output: {text}");
}

#[test]
fn unit_flags_scale_output() {
    for flag in ["-b", "-k", "-m", "-g"] {
        let out = cargo_bin_cmd!("free").arg(flag).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "flag {flag} must succeed");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("Mem"), "flag {flag}: {text}");
    }
}

#[test]
fn total_flag_adds_total_row() {
    let out = cargo_bin_cmd!("free").arg("-t").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Total"), "output: {text}");
}

#[test]
fn human_readable_uses_suffixes() {
    let out = cargo_bin_cmd!("free").arg("-h").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Gi") || text.contains("Mi"), "output: {text}");
}
