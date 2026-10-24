mod cli;

use chrono::Local;
use clap::Parser;
use cli::Args;
use ruxcmd_core::cpu::{self, CpuTimes};

const VERSION: &str = "top 0.1.0";

fn get_memory_info() -> (u64, u64, u64, u64) {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

    unsafe { GlobalMemoryStatusEx(&mut mem_status) };

    let total = mem_status.ullTotalPhys;
    let free = mem_status.ullAvailPhys;
    let used = total - free;

    (
        used / 1024 / 1024,
        free / 1024 / 1024,
        total / 1024 / 1024,
        mem_status.dwMemoryLoad as u64,
    )
}

/// Boot duration in seconds via GetTickCount64.
fn uptime_secs() -> u64 {
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    // SAFETY: GetTickCount64 takes no parameters and cannot fail.
    (unsafe { GetTickCount64() }) / 1000
}

fn warn_snapshot_failed_once() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SNAPSHOT_WARNED: AtomicBool = AtomicBool::new(false);
    if !SNAPSHOT_WARNED.swap(true, Ordering::Relaxed) {
        eprintln!("top: failed to create a process snapshot; process counts are unavailable");
    }
}

fn get_process_count() -> u64 {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        warn_snapshot_failed_once();
        return 0;
    }

    let mut count = 0u64;

    let mut pe: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    unsafe {
        let first = Process32FirstW(snapshot, &mut pe);
        if first != 0 {
            count += 1;
            loop {
                pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                let next = Process32NextW(snapshot, &mut pe);
                if next == 0 {
                    break;
                }
                count += 1;
            }
        }
        CloseHandle(snapshot);
    }

    count
}

fn header_process_count(processes: u64) -> u64 {
    processes.saturating_sub(1)
}

fn print_header(prev: &CpuTimes) {
    let now = Local::now();
    let (used, free, total, percent) = get_memory_info();
    let processes = get_process_count();
    let up = uptime_secs();
    let up_days = up / 86400;
    let up_hm = format!("{}:{:02}", (up % 86400) / 3600, (up % 3600) / 60);

    // CPU percentages cover the interval since the previous frame; the first
    // frame falls back to the since-boot average of a single sample.
    let (us, sy, id) = cpu::sample()
        .and_then(|cur| cpu::usage_between(prev, &cur))
        .or_else(|| cpu::sample().and_then(|cur| cpu::usage_between(&CpuTimes::ZERO, &cur)))
        .unwrap_or((0.0, 0.0, 0.0));

    println!("{}", "=".repeat(80));
    println!(
        "top - {} up {} days, {}, 0 users",
        now.format("%H:%M:%S"),
        up_days,
        up_hm
    );
    println!(
        "Tasks: {} total, 0 running, {} sleeping, 0 stopped, 0 zombie",
        processes,
        header_process_count(processes)
    );
    println!(
        "%Cpu(s): {:.1} us, {:.1} sy, 0.0 ni, {:.1} id, 0.0 wa, 0.0 hi, 0.0 si, 0.0 st",
        us, sy, id
    );
    println!(
        "KiB Mem : {:>10} total, {:>10} free, {:>10} used, {:>10} buff/cache",
        total * 1024,
        free * 1024,
        used * 1024,
        0
    );
    println!(
        "KiB Swap: {:>10} total, {:>10} free, {:>10} used. {:>10} avail Mem",
        0,
        0,
        0,
        free * 1024
    );
    println!("{}", "-".repeat(80));
}

fn print_process_header() {
    println!(
        "{:<6} {:<12} {:>6} {:>6} {:>10} {:>10} {:>6} {:<4} {}",
        "PID", "USER", "%CPU", "%MEM", "VIRT", "RES", "SHR", "S", "COMMAND"
    );
}

fn print_processes() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return;
    }

    let mut pe: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut count = 0;
    unsafe {
        let first = Process32FirstW(snapshot, &mut pe);
        if first != 0 {
            loop {
                let name = String::from_utf16_lossy(&pe.szExeFile);
                let name = name.trim_end_matches('\0');

                println!(
                    "{:<6} {:<12} {:>6} {:>6} {:>10} {:>10} {:>6} {:<4} {}",
                    pe.th32ProcessID, "SYSTEM", 0, 0, 0, 0, 0, "S", name
                );

                count += 1;
                if count >= 20 {
                    break;
                }

                pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                let next = Process32NextW(snapshot, &mut pe);
                if next == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let mut prev_cpu = cpu::sample().unwrap_or(CpuTimes::ZERO);
    for i in 0..args.iterations {
        print_header(&prev_cpu);
        if let Some(cur) = cpu::sample() {
            prev_cpu = cur;
        }
        print_process_header();
        print_processes();
        println!("{}", "=".repeat(80));

        if i < args.iterations - 1 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
            println!("\n\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::header_process_count;

    #[test]
    fn zero_processes_does_not_underflow() {
        // P0 #38 regression: an empty snapshot used to compute 0 - 1 and
        // wrap around to u64::MAX in release builds (debug panic).
        assert_eq!(header_process_count(0), 0);
    }

    #[test]
    fn nonzero_processes_keep_minus_one() {
        assert_eq!(header_process_count(1), 0);
        assert_eq!(header_process_count(5), 4);
        assert_eq!(header_process_count(u64::MAX), u64::MAX - 1);
    }
}
