mod cli;

use clap::Parser;
use cli::Cli;
use std::fs;
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    let path = Path::new(&cli.file);
    let result = if cli.canonicalize {
        fs::canonicalize(path)
    } else {
        fs::read_link(path)
    };

    match result {
        Ok(target) => {
            if let Some(target_str) = target.to_str() {
                if cli.no_newline {
                    print!("{}", target_str);
                } else {
                    println!("{}", target_str);
                }
            } else {
                eprintln!("readlink: cannot convert target path to string");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("readlink: {}: {}", cli.file, e);
            std::process::exit(1);
        }
    }
}
