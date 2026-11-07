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
    // Drive counters follow the same rhythm: the first report is the
    // since-boot average, later ones cover the interval between snapshots.
    let mut prev_drives = if delta_mode {
        ruxcmd_core::disk::snapshot()
    } else {
        Vec::new()
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
        let cur_drives = ruxcmd_core::disk::snapshot();
        let usage = usage_between(&prev, &cur)
            .or(last)
            .unwrap_or_else(|| fail_get_system_times());
        last = Some(usage);
        if args.device {
            // sysstat iostat -d prints the device report only.
            let interval = if delta_mode && printed > 0 {
                args.delay as f64
            } else {
                // Since-boot average: the machine's uptime stands in for the
                // interval (GetTickCount64, no wraparound for ~584 days).
                let ms = unsafe { getTickCount64() } / 1000;
                ms.max(1) as f64
            };
            print_device_report(&prev_drives, &cur_drives, interval);
            prev_drives = cur_drives;
        } else {
            print_report(&usage);
        }
        if delta_mode {
            prev = cur;
        }
        printed += 1;
    }
}

/// sysstat iostat -d column layout.  Windows DISK_PERFORMANCE has no discard
/// counters, so the kB_dscd columns stay zero as a declared difference.
fn print_device_report(
    prev: &[ruxcmd_core::disk::DriveStats],
    cur: &[ruxcmd_core::disk::DriveStats],
    interval: f64,
) {
    println!(
        "{:<14}{:>9}{:>13}{:>13}{:>13}{:>11}{:>11}{:>11}",
        "Device", "tps", "kB_read/s", "kB_wrtn/s", "kB_dscd/s", "kB_read", "kB_wrtn", "kB_dscd"
    );
    let interval = interval.max(1.0);
    for drive in cur {
        let prev_stats = prev.iter().find(|p| p.name == drive.name);
        let (reads, writes, read_bytes, write_bytes) = match prev_stats {
            Some(p) => (
                drive.read_count.saturating_sub(p.read_count),
                drive.write_count.saturating_sub(p.write_count),
                drive.read_bytes.saturating_sub(p.read_bytes),
                drive.write_bytes.saturating_sub(p.write_bytes),
            ),
            None => (
                drive.read_count,
                drive.write_count,
                drive.read_bytes,
                drive.write_bytes,
            ),
        };
        let tps = (reads + writes) as f64 / interval;
        println!(
            "{:<14}{:>9.2}{:>13.2}{:>13.2}{:>13.2}{:>11.0}{:>11.0}{:>11.0}",
            drive.name,
            tps,
            read_bytes as f64 / 1024.0 / interval,
            write_bytes as f64 / 1024.0 / interval,
            0.0,
            read_bytes as f64 / 1024.0,
            write_bytes as f64 / 1024.0,
            0.0
        );
    }
    println!();
}

/// GetTickCount64 from kernel32; the uptime in milliseconds.
#[cfg(windows)]
#[allow(non_snake_case)]
unsafe fn getTickCount64() -> u64 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetTickCount64() -> u64;
    }
    unsafe { GetTickCount64() }
}

#[cfg(not(windows))]
#[allow(non_snake_case)]
unsafe fn getTickCount64() -> u64 {
    1
}
