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
use ruxcmd_core::cpu::{self, CpuTimes};

const VERSION: &str = "iostat 0.1.0";

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
        cpu::sample().unwrap_or_else(|| fail_get_system_times())
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
        let cur = cpu::sample().unwrap_or_else(|| fail_get_system_times());
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
