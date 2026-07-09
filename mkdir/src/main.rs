mod cli;

use clap::Parser;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.directories.is_empty() {
        eprintln!("mkdir: missing operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for dir in &args.directories {
        if let Err(e) = mkdir_dir(dir, args.parents, args.verbose) {
            eprintln!("mkdir: cannot create directory '{}': {}", dir.display(), e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn mkdir_dir(dir: &Path, parents: bool, verbose: bool) -> Result<(), String> {
    if parents {
        match fs::create_dir_all(dir) {
            Ok(_) => {
                if verbose {
                    println!("mkdir: created directory '{}'", dir.display());
                }
                Ok(())
            }
            Err(e) => Err(format!("{}", e)),
        }
    } else {
        match fs::create_dir(dir) {
            Ok(_) => {
                if verbose {
                    println!("mkdir: created directory '{}'", dir.display());
                }
                Ok(())
            }
            Err(e) => Err(format!("{}", e)),
        }
    }
}