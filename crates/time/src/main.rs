mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;
use std::time::Instant;

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("time: missing command");
        std::process::exit(1);
    }

    let program = &args.command[0];
    let args: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let start = Instant::now();

    let status = Command::new(program)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("time: failed to execute '{}': {}", program, e);
            std::process::exit(1);
        });

    let elapsed = start.elapsed();

    println!("\nreal\t{:.3}s", elapsed.as_secs_f64());
    println!("user\t0.000s");
    println!("sys\t0.000s");

    std::process::exit(status.code().unwrap_or(1));
}
