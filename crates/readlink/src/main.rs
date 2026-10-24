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
            let target_str = if cli.canonicalize {
                ruxcmd_core::paths::display(&target)
            } else {
                match target.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        eprintln!("readlink: cannot convert target path to string");
                        std::process::exit(1);
                    }
                }
            };
            if cli.no_newline {
                print!("{}", target_str);
            } else {
                println!("{}", target_str);
            }
        }
        Err(e) => {
            eprintln!("readlink: {}: {}", cli.file, e);
            std::process::exit(1);
        }
    }
}
