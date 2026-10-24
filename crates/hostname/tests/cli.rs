//! Integration tests for hostname: failure reporting and normal output.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_hostname")
}

#[test]
fn prints_hostname_successfully() {
    let out = Command::new(bin()).output().unwrap();
    assert!(out.status.success(), "code: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.trim().is_empty(), "stdout: {:?}", stdout);
    assert!(
        out.stderr.is_empty(),
        "stderr: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn missing_hostname_reports_error_and_exits_one() {
    // Without COMPUTERNAME the lookup fails; the failure must not be silent.
    let out = Command::new(bin())
        .env_remove("COMPUTERNAME")
        .env_remove("HOSTNAME")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "code: {:?}", out.status.code());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.trim().is_empty(), "stderr: {:?}", stderr);
}
