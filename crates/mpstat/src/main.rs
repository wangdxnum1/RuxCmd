mod cli;

use clap::Parser;
use cli::Args;

const VERSION: &str = "mpstat 0.1.0";

fn get_cpu_count() -> u32 {
    num_cpus::get() as u32
}

fn print_header() {
    println!(
        "{:<5} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "CPU", "%usr", "%nice", "%sys", "%iowait", "%irq", "%soft", "%steal", "%idle"
    );
}

fn print_cpu_stats(cpu_index: u32) {
    println!(
        "{:<5} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        cpu_index, 0, 0, 0, 0, 0, 0, 0, 0
    );
}

fn main() {
    let args = Args::parse();

    if args.get_version() {
        println!("{}", VERSION);
        return;
    }

    let cpu_count = get_cpu_count();

    for i in 0..args.count {
        print_header();

        match &args.cpu {
            Some(cpu_str) if cpu_str == "ALL" => {
                for cpu in 0..cpu_count {
                    print_cpu_stats(cpu);
                }
            }
            Some(cpu_str) => {
                if let Ok(cpu) = cpu_str.parse::<u32>() {
                    print_cpu_stats(cpu);
                }
            }
            None => {
                print_cpu_stats(0);
            }
        }

        if i < args.count - 1 && args.delay > 0 {
            std::thread::sleep(std::time::Duration::from_secs(args.delay as u64));
        }
    }
}
