mod cli;

use clap::Parser;
use cli::Args;

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
    fn GetProcessId(hProcess: *mut std::ffi::c_void) -> u32;
    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> *mut std::ffi::c_void;
}

struct MemoryInfo {
    total_phys: u64,
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
        total_phys: mem_status.ullTotalPhys,
        avail_phys: mem_status.ullAvailPhys,
        total_pagefile: mem_status.ullTotalPageFile,
        avail_pagefile: mem_status.ullAvailPageFile,
        memory_load: mem_status.dwMemoryLoad,
    })
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

fn print_stats(mem_info: &MemoryInfo) {
    let r = get_process_count() as i64;
    let swpd = (mem_info.total_pagefile - mem_info.avail_pagefile) / 1024;
    let free = mem_info.avail_phys / 1024;

    println!(
        "{:>4} {:>4} {:>8} {:>8} {:>8} {:>8} {:>6} {:>6} {:>8} {:>8} {:>6} {:>6} {:>4} {:>4} {:>4} {:>4} {:>4}",
        r,
        0,
        swpd,
        free,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        100 - mem_info.memory_load,
        0,
        0
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let mem_info = get_memory_info().expect("vmstat: cannot access memory information");

    if !args.header_once || args.count == 1 {
        print_header();
    }

    for i in 0..args.count {
        if args.header_once && i > 0 {
        } else if i > 0 {
            print_header();
        }
        print_stats(&mem_info);

        if i < args.count - 1 && args.delay > 0 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
    }
}
