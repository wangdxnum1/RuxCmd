mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;

fn main() {
    let args = Args::parse();

    let zipfile = args.zipfile.to_string_lossy();
    let files: Vec<&str> = args
        .files
        .iter()
        .map(|p| p.to_str().unwrap_or(""))
        .collect();

    let recurse_flag = if args.recurse { "-r" } else { "" };

    let result = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Compress-Archive {} -Path {} -DestinationPath {} -Force",
                recurse_flag,
                files.join(", "),
                zipfile
            ),
        ])
        .output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            std::process::exit(output.status.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("zip: {}", e);
            std::process::exit(1);
        }
    }
}
