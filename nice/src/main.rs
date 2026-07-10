mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("nice: missing command");
        std::process::exit(1);
    }

    let program = &args.command[0];
    let cmd_args: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let priority = 8 + args.adjustment;
    let priority = priority.clamp(0, 15);

    let result = Command::new("wmic")
        .args(["process", "call", "create", &format!("cmd /c {}", program), &format!("priority={}", priority)])
        .output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            let mut child = Command::new(program)
                .args(cmd_args)
                .spawn()
                .unwrap_or_else(|e2| {
                    eprintln!("nice: failed to execute '{}': {}", program, e2);
                    std::process::exit(1);
                });
            let status = child.wait().unwrap();
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}