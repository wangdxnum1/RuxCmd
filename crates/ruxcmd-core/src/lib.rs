//! Shared helpers for RuxCmd commands.
//!
//! Each module consolidates a pattern that was previously copy-pasted across
//! many command crates during the P0/P1 remediation rounds:
//!
//! - [`lines`]: byte-safe line reading (CRLF-aware, non-UTF-8 safe)
//! - [`paths`]: Windows verbatim-path normalization for display
//! - [`cpu`]: GetSystemTimes sampling and percentage deltas
//! - [`fsutil`]: clearing the read-only attribute before overwrites

pub mod cpu;
pub mod fsutil;
pub mod lines;
pub mod paths;
