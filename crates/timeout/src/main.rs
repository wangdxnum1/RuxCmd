mod cli;

use clap::Parser;
use cli::Args;
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::Duration;

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("timeout: missing command");
        std::process::exit(1);
    }

    let duration = parse_duration(&args.duration);

    let program = &args.command[0];
    let cmd_args: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let mut child = Command::new(program)
        .args(cmd_args)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("timeout: failed to execute '{}': {}", program, e);
            std::process::exit(1);
        });

    let pid = child.id();

    let handle = thread::spawn(move || {
        thread::sleep(duration);
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output();
    });

    let status = child.wait().unwrap();

    handle.join().unwrap();

    if status.code().is_none() {
        eprintln!("timeout: command timed out");
        std::process::exit(124);
    }

    std::process::exit(status.code().unwrap());
}

fn parse_duration(s: &str) -> Duration {
    let s = s.to_lowercase();
    if s.ends_with('s') {
        let num: u64 = s[..s.len() - 1].parse().unwrap_or(0);
        Duration::from_secs(num)
    } else if s.ends_with('m') {
        let num: u64 = s[..s.len() - 1].parse().unwrap_or(0);
        Duration::from_secs(num * 60)
    } else if s.ends_with('h') {
        let num: u64 = s[..s.len() - 1].parse().unwrap_or(0);
        Duration::from_secs(num * 3600)
    } else {
        let num: u64 = s.parse().unwrap_or(0);
        Duration::from_secs(num)
    }
}
