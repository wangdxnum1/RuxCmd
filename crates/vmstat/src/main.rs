mod cli;

use clap::Parser;
use cli::Args;
use ruxcmd_core::cpu;

const VERSION: &str = "vmstat 0.1.0";

#[cfg(windows)]
#[repr(C)]
struct MEMORYSTATUSEX {
    dwLength: u32,
    dwMemoryLoad: u32,
    ullTotalPhys: u64,
    ullAvailPhys: u64,
    ullTotalPageFile: u64,
    ullAvailPageFile: u64,
    ullTotalVirtual: u64,
    ullAvailVirtual: u64,
    ullAvailExtendedVirtual: u64,
}

#[cfg(windows)]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
}

struct MemoryInfo {
    avail_phys: u64,
    total_pagefile: u64,
    avail_pagefile: u64,
    memory_load: u32,
}

fn get_memory_info() -> Option<MemoryInfo> {
    let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

    let result = unsafe { GlobalMemoryStatusEx(&mut mem_status) };

    if result == 0 {
        return None;
    }

    Some(MemoryInfo {
        avail_phys: mem_status.ullAvailPhys,
        total_pagefile: mem_status.ullTotalPageFile,
        avail_pagefile: mem_status.ullAvailPageFile,
        memory_load: mem_status.dwMemoryLoad,
    })
}

/// Take one memory snapshot, aborting when the OS refuses to provide one.
fn sample_memory() -> MemoryInfo {
    get_memory_info().expect("vmstat: cannot access memory information")
}

fn print_header() {
    println!(
        "{:>4} {:>4} {:>8} {:>8} {:>8} {:>8} {:>6} {:>6} {:>8} {:>8} {:>6} {:>6} {:>4} {:>4} {:>4} {:>4} {:>4}",
        "r",
        "b",
        "swpd",
        "free",
        "buff",
        "cache",
        "si",
        "so",
        "bi",
        "bo",
        "in",
        "cs",
        "us",
        "sy",
        "id",
        "wa",
        "st"
    );
}

/// us/sy/id percentages between two samples, truncated to integers from the
/// shared sampler's f64 result (vmstat prints whole percent columns).
fn cpu_percentages(prev: &cpu::CpuTimes, cur: &cpu::CpuTimes) -> Option<(u64, u64, u64)> {
    let (us, sy, id) = cpu::usage_between(prev, cur)?;
    Some((us as u64, sy as u64, id as u64))
}

fn print_stats(mem_info: &MemoryInfo, cpu: Option<(u64, u64, u64)>) {
    let swpd = (mem_info.total_pagefile - mem_info.avail_pagefile) / 1024;
    let free = mem_info.avail_phys / 1024;
    let (us, sy, id) = cpu.unwrap_or((0, 0, 0));

    // r has no Windows counterpart (runnable-task count) and prints 0.
    println!(
        "{:>4} {:>4} {:>8} {:>8} {:>8} {:>8} {:>6} {:>6} {:>8} {:>8} {:>6} {:>6} {:>4} {:>4} {:>4} {:>4} {:>4}",
        0, 0, swpd, free, 0, 0, 0, 0, 0, 0, 0, 0, us, sy, id, 0, 0
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let mut mem_info = sample_memory();
    let mut prev_cpu = cpu::sample();

    if !args.header_once || args.count == 1 {
        print_header();
    }

    for i in 0..args.count {
        if args.header_once && i > 0 {
        } else if i > 0 {
            print_header();
        }
        // CPU percentages cover the interval since the previous frame; the
        // first frame has no baseline yet and reports zeros.
        let cpu = prev_cpu
            .zip(cpu::sample())
            .and_then(|(p, c)| cpu_percentages(&p, &c));
        if let Some(cur) = cpu::sample() {
            prev_cpu = Some(cur);
        }
        print_stats(&mem_info, cpu);

        if i < args.count - 1 {
            if args.delay > 0 {
                std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
            }
            // Re-sample every frame: swpd/free must reflect the state at the
            // time each line is printed, not the state at startup (P0 #46).
            mem_info = sample_memory();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two consecutive samples must return numerically valid field values
    /// (types are checked by the compiler; ranges are checked here).
    #[test]
    fn consecutive_samples_return_valid_numeric_fields() {
        let first = get_memory_info().expect("memory information available");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let second = get_memory_info().expect("memory information available");

        for (label, info) in [("first", &first), ("second", &second)] {
            assert!(info.avail_phys > 0, "{label}: avail_phys must be positive");
            assert!(
                info.avail_pagefile <= info.total_pagefile,
                "{label}: avail_pagefile exceeds total_pagefile"
            );
            assert!(info.memory_load <= 100, "{label}: memory_load out of range");
        }
    }
}
