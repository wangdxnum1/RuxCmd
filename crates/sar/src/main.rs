//! sar clone for Windows (real-time mode only; no sa history files).
//!
//! CPU rows use real data from `GetSystemTimes` deltas, memory rows use
//! `GlobalMemoryStatusEx`, and every row carries a local HH:MM:SS timestamp.
//! The columns Windows cannot fill (%iowait, %irq) stay at 0.00 as a declared
//! difference. There is no lightweight per-disk API without PDH, so `-d` is
//! not declared instead of printing fabricated zeros.

mod cli;

use clap::Parser;
use cli::Args;
use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::System::SystemInformation::{
    GetLocalTime, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};
use windows_sys::Win32::System::Threading::GetSystemTimes;

const VERSION: &str = "sar 0.1.0";

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
    eprintln!("sar: GetSystemTimes failed");
    std::process::exit(1);
}

/// Memory snapshot in MiB plus the percent-used load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MemSample {
    used_mb: u64,
    free_mb: u64,
    total_mb: u64,
    percent: u64,
}

fn mem_sample_from(total_phys: u64, avail_phys: u64, load_percent: u64) -> MemSample {
    MemSample {
        used_mb: (total_phys - avail_phys) / 1024 / 1024,
        free_mb: avail_phys / 1024 / 1024,
        total_mb: total_phys / 1024 / 1024,
        percent: load_percent,
    }
}

fn sample_memory() -> Option<MemSample> {
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    // SAFETY: the struct is zeroed and its dwLength field is set as required.
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok == 0 {
        None
    } else {
        Some(mem_sample_from(
            status.ullTotalPhys,
            status.ullAvailPhys,
            status.dwMemoryLoad as u64,
        ))
    }
}

fn fail_memory() -> ! {
    eprintln!("sar: GlobalMemoryStatusEx failed");
    std::process::exit(1);
}

/// Local wall-clock time as HH:MM:SS for the leading column of each row.
fn local_hms() -> String {
    let mut now = SYSTEMTIME::default();
    // SAFETY: the out-parameter points to valid SYSTEMTIME storage.
    unsafe { GetLocalTime(&mut now) };
    format!("{:02}:{:02}:{:02}", now.wHour, now.wMinute, now.wSecond)
}

fn print_cpu_header() {
    println!(
        "{:<10}{:<6}{:>6}{:>7}{:>7}{:>8}{:>6}",
        "", "CPU", "%usr", "%sys", "%idle", "%iowait", "%irq"
    );
}

/// %iowait and %irq have no Windows counterpart (0.00).
fn print_cpu_row(label: &str, u: &CpuUsage) {
    println!(
        "{:<10}{:<6}{:>6.2}{:>7.2}{:>7.2}{:>8.2}{:>6.2}",
        label, "all", u.usr, u.sys, u.idle, 0.0, 0.0
    );
}

fn print_mem_header() {
    println!(
        "{:<10}{:<6}{:>10}{:>10}{:>10}{:>7}",
        "", "MB", "used", "free", "total", "%used"
    );
}

fn print_mem_row(label: &str, m: &MemSample) {
    println!(
        "{:<10}{:<6}{:>10}{:>10}{:>10}{:>7}",
        label, "Mem", m.used_mb, m.free_mb, m.total_mb, m.percent
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let show_cpu = args.cpu || (!args.cpu && !args.memory);
    let show_memory = args.memory || (!args.cpu && !args.memory);

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
    let mut cpu_header_shown = false;
    let mut mem_header_shown = false;
    let mut cpu_usages: Vec<CpuUsage> = Vec::new();
    let mut mem_samples: Vec<MemSample> = Vec::new();
    let mut printed: u64 = 0;

    loop {
        if frames.is_some_and(|n| printed >= n) {
            break;
        }
        // The interval separates the two CPU samples: sleep before re-sampling
        // so delta mode never measures a zero-length interval.
        if delta_mode {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
        if show_cpu {
            let cur = sample_cpu_times().unwrap_or_else(|| fail_get_system_times());
            let usage = usage_between(&prev, &cur)
                .or(last)
                .unwrap_or_else(|| fail_get_system_times());
            last = Some(usage);
            if !cpu_header_shown {
                print_cpu_header();
                cpu_header_shown = true;
            }
            print_cpu_row(&local_hms(), &usage);
            cpu_usages.push(usage);
            if delta_mode {
                prev = cur;
            }
        }
        if show_memory {
            let sample = sample_memory().unwrap_or_else(|| fail_memory());
            if !mem_header_shown {
                print_mem_header();
                mem_header_shown = true;
            }
            print_mem_row(&local_hms(), &sample);
            mem_samples.push(sample);
        }
        printed += 1;
    }

    // Like sysstat, the trailing Average rows only appear for finite runs.
    if args.count.is_some() {
        if !cpu_usages.is_empty() {
            let n = cpu_usages.len() as f64;
            let mean = cpu_usages.iter().fold((0.0, 0.0, 0.0), |acc, u| {
                (acc.0 + u.usr, acc.1 + u.sys, acc.2 + u.idle)
            });
            print_cpu_row(
                "Average:",
                &CpuUsage {
                    usr: mean.0 / n,
                    sys: mean.1 / n,
                    idle: mean.2 / n,
                },
            );
        }
        if !mem_samples.is_empty() {
            let n = mem_samples.len() as f64;
            let mean = mem_samples.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, m| {
                (
                    acc.0 + m.used_mb as f64,
                    acc.1 + m.free_mb as f64,
                    acc.2 + m.total_mb as f64,
                    acc.3 + m.percent as f64,
                )
            });
            let round = |v: f64| (v / n).round() as u64;
            print_mem_row(
                "Average:",
                &MemSample {
                    used_mb: round(mean.0),
                    free_mb: round(mean.1),
                    total_mb: round(mean.2),
                    percent: round(mean.3),
                },
            );
        }
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

    #[test]
    fn memory_sample_mib_math() {
        const GB: u64 = 1024 * 1024 * 1024;
        let m = mem_sample_from(8 * GB, 2 * GB + 512 * 1024 * 1024, 81);
        assert_eq!(m.total_mb, 8192);
        assert_eq!(m.free_mb, 2560);
        assert_eq!(m.used_mb, 8192 - 2560);
        assert_eq!(m.percent, 81);
    }
}
