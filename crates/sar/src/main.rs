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
use ruxcmd_core::cpu::{self, CpuTimes};
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::{
    GetLocalTime, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};

const VERSION: &str = "sar 0.1.0";

/// CPU usage percentages over one interval.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CpuUsage {
    usr: f64,
    sys: f64,
    idle: f64,
}

/// usr/sys/idle percentages for the interval between two samples, taken from
/// the shared sampler. Passing `CpuTimes::ZERO` as `prev` yields the
/// since-boot averages of a single sample. Returns `None` when no time
/// elapsed between the samples or the counters moved backwards.
fn usage_between(prev: &CpuTimes, cur: &CpuTimes) -> Option<CpuUsage> {
    let (usr, sys, idle) = cpu::usage_between(prev, cur)?;
    Some(CpuUsage { usr, sys, idle })
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
        cpu::sample().unwrap_or_else(|| fail_get_system_times())
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
            let cur = cpu::sample().unwrap_or_else(|| fail_get_system_times());
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
