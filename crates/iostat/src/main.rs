//! iostat clone for Windows.
//!
//! The device section of sysstat's iostat is not implemented: there is no
//! lightweight API for per-disk I/O counters without PDH, and printing
//! fabricated zeros is worse than omitting the section. This tool reports the
//! avg-cpu section with real data from `GetSystemTimes` deltas; the columns
//! Windows cannot fill (%nice, %iowait, %steal) stay at 0.00 as a declared
//! difference.

mod cli;

use clap::Parser;
use cli::Args;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::GetSystemTimes;

const VERSION: &str = "iostat 0.1.0";

/// System CPU time counters in 100 ns units, as returned by `GetSystemTimes`.
/// Windows reports `kernel` inclusive of idle time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CpuTimes {
    idle: u64,
    kernel: u64,
    user: u64,
}

impl CpuTimes {
    const ZERO: CpuTimes = CpuTimes {
        idle: 0,
        kernel: 0,
        user: 0,
    };

    fn from_filetimes(idle: FILETIME, kernel: FILETIME, user: FILETIME) -> Self {
        CpuTimes {
            idle: filetime_u64(&idle),
            kernel: filetime_u64(&kernel),
            user: filetime_u64(&user),
        }
    }
}

/// CPU usage percentages over one interval.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CpuUsage {
    usr: f64,
    sys: f64,
    idle: f64,
}

fn filetime_u64(ft: &FILETIME) -> u64 {
    (ft.dwHighDateTime as u64) << 32 | ft.dwLowDateTime as u64
}

/// usr/sys/idle percentages for the interval between two samples:
/// usr = ΔUser, sys = ΔKernel − ΔIdle, idle = ΔIdle, each over the total
/// ΔKernel + ΔUser. Passing `CpuTimes::ZERO` as `prev` yields the since-boot
/// averages of a single sample. Returns `None` when no time elapsed between
/// the samples or the counters moved backwards.
fn usage_between(prev: &CpuTimes, cur: &CpuTimes) -> Option<CpuUsage> {
    let d_user = cur.user.checked_sub(prev.user)?;
    let d_kernel = cur.kernel.checked_sub(prev.kernel)?;
    let d_idle = cur.idle.checked_sub(prev.idle)?;
    let d_sys = d_kernel.checked_sub(d_idle)?;
    let total = d_kernel.checked_add(d_user)?;
    if total == 0 {
        return None;
    }
    let pct = |v: u64| v as f64 * 100.0 / total as f64;
    Some(CpuUsage {
        usr: pct(d_user),
        sys: pct(d_sys),
        idle: pct(d_idle),
    })
}

fn sample_cpu_times() -> Option<CpuTimes> {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: all out-parameters point to valid FILETIME storage.
    let ok = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    if ok == 0 {
        None
    } else {
        Some(CpuTimes::from_filetimes(idle, kernel, user))
    }
}

fn fail_get_system_times() -> ! {
    eprintln!("iostat: GetSystemTimes failed");
    std::process::exit(1);
}

/// One report: the sysstat avg-cpu header with the six columns iostat keeps.
/// %nice, %iowait and %steal have no Windows counterpart and stay at 0.00.
fn print_report(usage: &CpuUsage) {
    println!(
        "{:<8}{:>7}{:>8}{:>8}{:>8}{:>8}{:>7}",
        "avg-cpu:", "%user", "%nice", "%system", "%iowait", "%steal", "%idle"
    );
    println!(
        "{:<8}{:>7.2}{:>8.2}{:>8.2}{:>8.2}{:>8.2}{:>7.2}",
        "", usage.usr, 0.0, usage.sys, 0.0, 0.0, usage.idle
    );
    println!();
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    // Frames to print: an explicit count, endless sampling when only a delay
    // is given (sysstat behaviour; Ctrl+C stops it), or a single since-boot
    // report when no interval is given at all.
    let frames: Option<u64> = match args.count {
        Some(count) => Some(count as u64),
        None if args.delay > 0 => None,
        None => Some(1),
    };

    // With a delay, percentages cover the interval between samples; without
    // one, each report is the since-boot average of a single sample.
    let delta_mode = args.delay > 0;
    let mut prev = if delta_mode {
        sample_cpu_times().unwrap_or_else(|| fail_get_system_times())
    } else {
        CpuTimes::ZERO
    };
    let mut last: Option<CpuUsage> = None;
    let mut printed: u64 = 0;

    loop {
        if frames.is_some_and(|n| printed >= n) {
            break;
        }
        // The interval separates the two samples: sleep before re-sampling so
        // delta mode never measures a zero-length interval.
        if delta_mode {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
        let cur = sample_cpu_times().unwrap_or_else(|| fail_get_system_times());
        let usage = usage_between(&prev, &cur)
            .or(last)
            .unwrap_or_else(|| fail_get_system_times());
        last = Some(usage);
        print_report(&usage);
        if delta_mode {
            prev = cur;
        }
        printed += 1;
    }
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

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn filetime_pairs_combine_to_u64() {
        assert_eq!(filetime_u64(&ft(1)), 1);
        assert_eq!(filetime_u64(&ft(u64::from(u32::MAX) + 2)), 0x1_0000_0001);
    }

    /// Two concrete FILETIME sample pairs: usr 50%, sys 16.67%, idle 33.33%.
    #[test]
    fn percentages_from_two_filetime_samples() {
        let a = CpuTimes::from_filetimes(ft(1_000_000), ft(3_000_000), ft(1_000_000));
        let b = CpuTimes::from_filetimes(ft(2_000_000), ft(4_500_000), ft(2_500_000));
        let u = usage_between(&a, &b).expect("non-empty interval");
        assert!(approx(u.usr, 50.0), "usr = {}", u.usr);
        assert!(approx(u.sys, 50.0 / 3.0), "sys = {}", u.sys);
        assert!(approx(u.idle, 100.0 / 3.0), "idle = {}", u.idle);
        assert!(approx(u.usr + u.sys + u.idle, 100.0));
    }

    #[test]
    fn zero_delta_or_backwards_counters_yield_none() {
        let t = CpuTimes {
            idle: 5,
            kernel: 6,
            user: 7,
        };
        assert!(usage_between(&t, &t).is_none());
        assert!(usage_between(&t, &CpuTimes::ZERO).is_none());
    }

    /// A zero baseline turns one sample into since-boot averages.
    #[test]
    fn zero_baseline_is_since_boot_average() {
        let t = CpuTimes {
            idle: 750,
            kernel: 1_000,
            user: 250,
        };
        let u = usage_between(&CpuTimes::ZERO, &t).expect("boot counters");
        assert!(approx(u.idle, 60.0));
        assert!(approx(u.usr, 20.0));
        assert!(approx(u.sys, 20.0));
    }
}
