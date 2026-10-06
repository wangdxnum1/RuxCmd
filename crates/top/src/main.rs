mod cli;

use chrono::Local;
use clap::Parser;
use cli::Args;

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

fn get_process_count() -> u64 {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
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

fn print_header() {
    let now = Local::now();
    let (used, free, total, percent) = get_memory_info();
    let processes = get_process_count();

    println!("{}", "=".repeat(80));
    println!(
        "top - {} up 0 days, 0:00, 0 users,  load average: 0.00, 0.00, 0.00",
        now.format("%H:%M:%S")
    );
    println!(
        "Tasks: {} total, 0 running, {} sleeping, 0 stopped, 0 zombie",
        processes,
        (processes - 1)
    );
    println!("%Cpu(s): 0.0 us, 0.0 sy, 0.0 ni, 0.0 id, 0.0 wa, 0.0 hi, 0.0 si, 0.0 st");
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

    for i in 0..args.iterations {
        print_header();
        print_process_header();
        print_processes();
        println!("{}", "=".repeat(80));

        if i < args.iterations - 1 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
            println!("\n\n");
        }
    }
}
