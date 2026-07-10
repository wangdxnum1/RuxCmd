mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;

fn main() {
    let args = Args::parse();

    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .expect("Failed to run tasklist");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut killed = false;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim_matches('"');
            let pid = parts[1].trim_matches('"');

            let mut matched = false;
            if args.full {
                let cmdline = get_cmdline(pid.parse().unwrap_or(0));
                if cmdline.contains(&args.pattern) {
                    matched = true;
                }
            } else {
                if name.to_lowercase().contains(&args.pattern.to_lowercase()) {
                    matched = true;
                }
            }

            if matched {
                let result = Command::new("taskkill")
                    .args(["/PID", pid, "/F"])
                    .output();
                match result {
                    Ok(_) => {
                        killed = true;
                    }
                    Err(e) => {
                        eprintln!("Failed to kill process {}: {}", pid, e);
                    }
                }
            }
        }
    }

    if !killed {
        std::process::exit(1);
    }
}

fn get_cmdline(pid: u32) -> String {
    let output = Command::new("wmic")
        .args(["process", "where", &format!("processid={}", pid), "get", "commandline", "/value"])
        .output();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("CommandLine=") {
                return line["CommandLine=".len()..].to_string();
            }
        }
    }
    String::new()
}