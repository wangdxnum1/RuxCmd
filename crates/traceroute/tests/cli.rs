//! Behavior tests for the Round 15 traceroute command.
//!
//! The binary is always spawned via the absolute `CARGO_BIN_EXE_traceroute`
//! path: cargo test prepends target/debug to PATH, so spawning a bare
//! `traceroute` could resolve to some other binary on PATH (same trap as the
//! ping crate's Round 12 A1 ledger note).
//!
//! Loopback and TEST-NET-3 targets keep the suite deterministic without a
//! real multi-hop route; intermediate-router behavior is not asserted.
//! The whole suite must stay well under 30 seconds, so the TEST-NET case
//! uses `-w 200` to shorten per-probe timeouts.

use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn traceroute(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_traceroute"))
        .args(args)
        .output()
        .expect("failed to run traceroute")
}

fn run_timed(args: &[&str]) -> (Output, Duration) {
    let start = Instant::now();
    let out = traceroute(args);
    (out, start.elapsed())
}

#[test]
fn loopback_reaches_target_at_first_hop() {
    let (out, elapsed) = run_timed(&["-w", "2000", "127.0.0.1"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Tracing route to"), "stdout: {stdout:?}");
    assert!(
        stdout.contains("127.0.0.1"),
        "header and hop line must carry the resolved IP: {stdout:?}"
    );
    // Hop 1 is the target itself on loopback, so the first hop line must
    // show the address plus numeric RTT probes instead of `*`.
    let hop1 = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("1 "))
        .unwrap_or_else(|| panic!("no hop 1 line in: {stdout:?}"));
    assert!(
        hop1.contains("127.0.0.1"),
        "first hop must be the target: {hop1:?}"
    );
    assert!(
        hop1.contains(" ms"),
        "first hop must report an RTT, got: {hop1:?}"
    );
    assert!(
        !hop1.contains('*'),
        "loopback probes must not time out: {hop1:?}"
    );
    assert!(elapsed < Duration::from_secs(15), "took {elapsed:?}");
}

#[test]
fn unreachable_testnet_target_terminates_without_hanging() {
    // 203.0.113.1 is TEST-NET-3 (RFC 5737): unroutable, guaranteed no reply.
    // With -w 200 the worst case is bounded well under 30s either through
    // the 3-consecutive-timeout early exit or fast unreachable replies.
    let (out, elapsed) = run_timed(&["-w", "200", "203.0.113.1"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Tracing route to 203.0.113.1"),
        "stdout: {stdout:?}"
    );
    assert!(
        elapsed < Duration::from_secs(30),
        "trace must not hang; took {elapsed:?}"
    );
}

#[test]
fn unresolvable_host_exits_one() {
    let out = traceroute(&["no-such-host-for-traceroute-test.invalid"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.is_empty(), "expected an error message on stderr");
}

#[test]
fn missing_host_prints_usage_and_exits_one() {
    let out = traceroute(&[]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Usage"), "stderr: {stderr:?}");
}
