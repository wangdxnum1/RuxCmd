mod cli;

use clap::Parser;
use cli::Args;

const VERSION: &str = "iostat 0.1.0";

fn get_disk_names() -> Vec<String> {
    let mut names = Vec::new();
    for c in b'A'..=b'Z' {
        let drive = format!("{}:", c as char);
        if std::path::Path::new(&drive).exists() {
            names.push(drive);
        }
    }
    names
}

fn get_disk_io(drive: &str) -> (u64, u64, u64) {
    (0, 0, 0)
}

fn print_header() {
    println!(
        "{:<12} {:>8} {:>12} {:>12} {:>12} {:>12}",
        "Device", "tps", "kB_read/s", "kB_wrtn/s", "kB_read", "kB_wrtn"
    );
}

fn print_disk_stats(disk_name: &str) {
    let (tps, read, write) = get_disk_io(disk_name);
    println!(
        "{:<12} {:>8} {:>12} {:>12} {:>12} {:>12}",
        disk_name, tps, read, write, 0, 0
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let disk_names = get_disk_names();

    for i in 0..args.count {
        print_header();
        for disk in &disk_names {
            print_disk_stats(disk);
        }

        if i < args.count - 1 && args.delay > 0 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
    }
}
