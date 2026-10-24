//! Integration tests for sleep: fractional durations and overflow rejection.

use std::process::Command;
use std::time::{Duration, Instant};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_sleep")
}

#[test]
fn fractional_duration_sleeps_briefly() {
    let start = Instant::now();
    let out = Command::new(bin()).arg("0.2").output().unwrap();
    assert!(out.status.success(), "code: {:?}", out.status.code());
    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(150),
        "should sleep at least ~0.2s, got {:?}",
        elapsed
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "0.2s must not sleep for {:?}",
        elapsed
    );
}

#[test]
fn invalid_number_is_rejected() {
    let out = Command::new(bin()).arg("xyz").output().unwrap();
    assert_ne!(out.status.code(), Some(0));
}

#[test]
fn overflow_is_rejected() {
    let out = Command::new(bin())
        .arg("99999999999999999999h")
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
}
