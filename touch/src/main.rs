mod cli;

use clap::Parser;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("touch: missing file operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for path in &args.files {
        if let Err(e) = touch_file(path, args.no_create) {
            eprintln!("touch: {}: {}", path.display(), e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn touch_file(path: &Path, no_create: bool) -> Result<(), String> {
    if path.exists() {
        match fs::OpenOptions::new().append(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("cannot open file: {}", e)),
        }
    } else {
        if no_create {
            Ok(())
        } else {
            match fs::File::create(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("cannot create file: {}", e)),
            }
        }
    }
}