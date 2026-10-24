//! Behavior tests for the Round 12 G1 ping P1 fixes.
//!
//! The binary is always spawned via the absolute `CARGO_BIN_EXE_ping` path:
//! cargo test prepends target/debug to PATH, so spawning a bare `ping` would
//! resolve to this crate itself (Round 12 A1 ledger note).
//!
//! Non-ignored tests stay network-free: the size limit is validated before
//! any host resolution or ICMP work. Tests that actually send ICMP echo
//! requests are #[ignore]d for manual acceptance.

use std::process::{Command, Output};

fn ping(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ping"))
        .args(args)
        .output()
        .expect("failed to run ping")
}

#[test]
fn size_over_limit_fails_before_any_network_use() {
    let out = ping(&["-s", "70000", "127.0.0.1"]);
    assert_eq!(out.status.code(), Some(1), "stderr: {:?}", out.stderr);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("valid range is 0 to 65500"),
        "stderr: {stderr:?}"
    );
}

#[test]
#[ignore = "requires ICMP; manual acceptance for Round 12 G1"]
fn loopback_ping_exits_zero() {
    let out = ping(&["-c", "1", "-w", "2000", "127.0.0.1"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Reply from 127.0.0.1"),
        "stdout: {stdout:?}"
    );
    assert!(stdout.contains("Lost = 0"), "stdout: {stdout:?}");
}

#[test]
#[ignore = "requires ICMP; pings TEST-NET-3 which must never answer"]
fn all_packets_lost_exits_one() {
    // 203.0.113.1 is TEST-NET-3 (RFC 5737): unroutable, guaranteed no reply.
    let out = ping(&["-c", "1", "-w", "1000", "203.0.113.1"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "total packet loss must exit 1; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Lost = 1"), "stdout: {stdout:?}");
    assert!(
        !stdout.contains("Approximate round trip times"),
        "no RTT stats without replies: {stdout:?}"
    );
}
