mod cli;

use clap::Parser;
use cli::{Args, Unit};

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

fn main() {
    let args = Args::parse();
    let unit = args.get_unit();

    let repeat = args.seconds.unwrap_or(0);
    let frames = match (args.seconds, args.count) {
        (Some(_), Some(c)) => c,
        (Some(_), None) => u64::MAX,
        (None, _) => 1,
    };

    for frame in 0..frames {
        if frame > 0 {
            std::thread::sleep(std::time::Duration::from_secs(repeat));
        }
        let Some(mem_info) = get_memory_info() else {
            eprintln!("free: cannot access memory information");
            std::process::exit(1);
        };
        print_free(&args, &mem_info, unit);
    }
}

fn print_free(args: &Args, mem_info: &MemoryInfo, unit: Unit) {
    let total = mem_info.total_phys;
    let used = total - mem_info.avail_phys;
    let free = mem_info.avail_phys;
    let total_swap = mem_info.total_pagefile;
    let used_swap = total_swap - mem_info.avail_pagefile;
    let free_swap = mem_info.avail_pagefile;

    if args.wide {
        // Wide format: available column instead of the placeholder columns.
        println!(
            "{:>8} {:>12} {:>12} {:>12} {:>12}",
            "", "total", "used", "free", "available"
        );
        println!(
            "{:>8} {:>12} {:>12} {:>12} {:>12}",
            "Mem:",
            format_size(total, unit),
            format_size(used, unit),
            format_size(free, unit),
            format_size(free, unit)
        );
    } else {
        println!(
            "{:>8} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "", "total", "used", "free", "shared", "buff/cache"
        );
        println!(
            "{:>8} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "Mem:",
            format_size(total, unit),
            format_size(used, unit),
            format_size(free, unit),
            "-",
            "-"
        );
    }

    println!(
        "{:>8} {:>12} {:>12} {:>12}",
        "Swap:",
        format_size(total_swap, unit),
        format_size(used_swap, unit),
        format_size(free_swap, unit)
    );

    if args.total {
        println!(
            "{:>8} {:>12} {:>12} {:>12}",
            "Total:",
            format_size(total + total_swap, unit),
            format_size(used + used_swap, unit),
            format_size(free + free_swap, unit)
        );
    }
}

struct MemoryInfo {
    total_phys: u64,
    avail_phys: u64,
    total_pagefile: u64,
    avail_pagefile: u64,
}

#[cfg(windows)]
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
    })
}

#[cfg(unix)]
fn get_memory_info() -> Option<MemoryInfo> {
    None
}

fn format_size(bytes: u64, unit: Unit) -> String {
    match unit {
        Unit::Bytes => bytes.to_string(),
        Unit::Kilobytes => (bytes / 1024).to_string(),
        Unit::Megabytes => (bytes / 1024 / 1024).to_string(),
        Unit::Gigabytes => (bytes / 1024 / 1024 / 1024).to_string(),
        Unit::Human => {
            let units = ["B", "Ki", "Mi", "Gi", "Ti"];
            let mut size = bytes as f64;
            let mut unit_idx = 0;

            while size >= 1024.0 && unit_idx < units.len() - 1 {
                size /= 1024.0;
                unit_idx += 1;
            }

            if unit_idx == 0 {
                format!("{:.0}", size)
            } else {
                format!("{:.1}{}", size, units[unit_idx])
            }
        }
    }
}
