mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;

fn main() {
    let _args = Args::parse();

    let result = Command::new("sync").output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => {
            let _ = Command::new("rundll32")
                .args(["advapi32.dll,ProcessIdleTasks"])
                .output();
        }
    }
}
