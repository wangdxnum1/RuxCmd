//! System CPU sampling via `GetSystemTimes`.
//!
//! Windows reports kernel time inclusive of idle time, so the busy split is
//! derived as `sys = kernel - idle` over the total `kernel + user`. Passing
//! `CpuTimes::ZERO` as the previous sample turns a single reading into the
//! since-boot average.

use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::GetSystemTimes;

/// System CPU counters in 100 ns units (kernel includes idle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    pub kernel: u64,
    pub user: u64,
}

impl CpuTimes {
    pub const ZERO: CpuTimes = CpuTimes {
        idle: 0,
        kernel: 0,
        user: 0,
    };
}

fn filetime_u64(ft: &FILETIME) -> u64 {
    (ft.dwHighDateTime as u64) << 32 | ft.dwLowDateTime as u64
}

/// One reading of the system CPU counters, or `None` if the call fails.
pub fn sample() -> Option<CpuTimes> {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: all out-parameters point to valid FILETIME storage.
    let ok = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    if ok == 0 {
        return None;
    }
    Some(CpuTimes {
        idle: filetime_u64(&idle),
        kernel: filetime_u64(&kernel),
        user: filetime_u64(&user),
    })
}

/// usr/sys/idle percentages for the interval between two samples.
/// `None` when no time elapsed or the counters moved backwards.
pub fn usage_between(prev: &CpuTimes, cur: &CpuTimes) -> Option<(f64, f64, f64)> {
    let d_user = cur.user.checked_sub(prev.user)?;
    let d_kernel = cur.kernel.checked_sub(prev.kernel)?;
    let d_idle = cur.idle.checked_sub(prev.idle)?;
    let d_sys = d_kernel.checked_sub(d_idle)?;
    let total = d_kernel.checked_add(d_user)?;
    if total == 0 {
        return None;
    }
    let pct = |v: u64| v as f64 * 100.0 / total as f64;
    Some((pct(d_user), pct(d_sys), pct(d_idle)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ft(v: u64) -> FILETIME {
        FILETIME {
            dwLowDateTime: v as u32,
            dwHighDateTime: (v >> 32) as u32,
        }
    }

    #[test]
    fn live_sample_has_positive_total() {
        let s = sample().expect("GetSystemTimes works");
        assert!(s.kernel + s.user > 0);
        assert!(s.idle <= s.kernel);
    }

    #[test]
    fn percentages_match_manual_deltas() {
        // 50% user, ~16.7% sys, ~33.3% idle.
        let a = CpuTimes {
            idle: 1_000_000,
            kernel: 3_000_000,
            user: 1_000_000,
        };
        let b = CpuTimes {
            idle: 2_000_000,
            kernel: 4_500_000,
            user: 2_500_000,
        };
        let (u, s, i) = usage_between(&a, &b).unwrap();
        assert!((u - 50.0).abs() < 0.01);
        assert!((s - 50.0 / 3.0).abs() < 0.01);
        assert!((i - 100.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn zero_interval_is_none() {
        let t = CpuTimes {
            idle: 5,
            kernel: 6,
            user: 7,
        };
        assert!(usage_between(&t, &t).is_none());
    }
}
