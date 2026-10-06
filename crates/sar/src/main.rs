mod cli;

use clap::Parser;
use cli::Args;

const VERSION: &str = "sar 0.1.0";

fn get_memory_info() -> (u64, u64, u64, u64, u64) {
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
        0,
    )
}

fn print_cpu_header() {
    println!(
        "{:<8} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "CPU", "%usr", "%sys", "%idle", "%iowait", "%irq"
    );
}

fn print_memory_header() {
    println!(
        "{:<12} {:>10} {:>10} {:>10} {:>6}",
        "MB", "used", "free", "total", "%used"
    );
}

fn print_memory_stats() {
    let (used, free, total, percent, _) = get_memory_info();
    println!(
        "{:<12} {:>10} {:>10} {:>10} {:>6}",
        "Mem", used, free, total, percent
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let show_cpu = args.cpu || (!args.cpu && !args.memory && !args.disk);
    let show_memory = args.memory || (!args.cpu && !args.memory && !args.disk);

    for i in 0..args.count {
        if show_cpu {
            print_cpu_header();
            println!("{:<8} {:>6} {:>6} {:>6} {:>6} {:>6}", "all", 0, 0, 0, 0, 0);
        }

        if show_memory {
            print_memory_header();
            print_memory_stats();
        }

        if i < args.count - 1 && args.delay > 0 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
    }
}
