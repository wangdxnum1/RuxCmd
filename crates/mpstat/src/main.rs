//! mpstat clone for Windows.
//!
//! Windows has no lightweight per-CPU counters without PDH, so only the
//! aggregate "all" row is reported, with real data from `GetSystemTimes`
//! deltas. The columns Windows cannot fill (%nice, %iowait, %irq, %soft,
//! %steal) stay at 0.00 as a declared difference, matching the kept sysstat
//! header.

mod cli;

use clap::Parser;
use cli::Args;
use ruxcmd_core::cpu::{self, CpuTimes};

const VERSION: &str = "mpstat 0.1.0";

/// CPU usage percentages over one interval.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CpuUsage {
    usr: f64,
    sys: f64,
    idle: f64,
}

impl CpuUsage {
    const ZERO_USAGE: CpuUsage = CpuUsage {
        usr: 0.0,
        sys: 0.0,
        idle: 0.0,
    };
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
    eprintln!("mpstat: GetSystemTimes failed");
    std::process::exit(1);
}

/// Row layout: label column plus the nine kept sysstat columns. %nice,
/// %iowait, %irq, %soft and %steal have no Windows counterpart (0.00).
fn print_row(label: &str, u: &CpuUsage) {
    println!(
        "{:<8}{:>7.2}{:>7.2}{:>7.2}{:>8.2}{:>7.2}{:>7.2}{:>7.2}{:>7.2}",
        label, u.usr, 0.0, u.sys, 0.0, 0.0, 0.0, 0.0, u.idle
    );
}

fn print_header() {
    println!(
        "{:<8}{:>7}{:>7}{:>7}{:>8}{:>7}{:>7}{:>7}{:>7}",
        "CPU", "%usr", "%nice", "%sys", "%iowait", "%irq", "%soft", "%steal", "%idle"
    );
}

fn average(usages: &[CpuUsage]) -> CpuUsage {
    let n = usages.len() as f64;
    let sum = usages.iter().fold(CpuUsage::ZERO_USAGE, |acc, u| CpuUsage {
        usr: acc.usr + u.usr,
        sys: acc.sys + u.sys,
        idle: acc.idle + u.idle,
    });
    CpuUsage {
        usr: sum.usr / n,
        sys: sum.sys / n,
        idle: sum.idle / n,
    }
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    // Only the aggregate row exists: Windows per-CPU counters would need PDH.
    if let Some(cpu) = &args.cpu {
        if !cpu.eq_ignore_ascii_case("ALL") {
            eprintln!("mpstat: per-cpu output is not supported");
            std::process::exit(1);
        }
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
        cpu::sample().unwrap_or_else(|| fail_get_system_times())
    } else {
        CpuTimes::ZERO
    };
    let mut last: Option<CpuUsage> = None;
    let mut usages: Vec<CpuUsage> = Vec::new();
    let mut printed: u64 = 0;
    let mut header_shown = false;

    loop {
        if frames.is_some_and(|n| printed >= n) {
            break;
        }
        // The interval separates the two samples: sleep before re-sampling so
        // delta mode never measures a zero-length interval.
        if delta_mode {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
        let cur = cpu::sample().unwrap_or_else(|| fail_get_system_times());
        let usage = usage_between(&prev, &cur)
            .or(last)
            .unwrap_or_else(|| fail_get_system_times());
        last = Some(usage);
        if !header_shown {
            print_header();
            header_shown = true;
        }
        print_row("all", &usage);
        usages.push(usage);
        if delta_mode {
            prev = cur;
        }
        printed += 1;
    }

    // Like sysstat, the trailing Average row only appears for finite runs.
    if args.count.is_some() && !usages.is_empty() {
        print_row("Average:", &average(&usages));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn average_is_the_column_mean() {
        let usages = [
            CpuUsage {
                usr: 10.0,
                sys: 5.0,
                idle: 85.0,
            },
            CpuUsage {
                usr: 30.0,
                sys: 10.0,
                idle: 60.0,
            },
        ];
        let avg = average(&usages);
        assert!(approx(avg.usr, 20.0));
        assert!(approx(avg.sys, 7.5));
        assert!(approx(avg.idle, 72.5));
    }
}
