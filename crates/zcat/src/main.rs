use std::fs::File;
use std::io::{self};
use std::process;

use clap::Parser;
use flate2::read::GzDecoder;

mod cli;

fn decompress_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut stdout = io::stdout();
    io::copy(&mut decoder, &mut stdout)?;
    Ok(())
}

fn decompress_stdin() -> io::Result<()> {
    let mut decoder = GzDecoder::new(io::stdin());
    let mut stdout = io::stdout();
    io::copy(&mut decoder, &mut stdout)?;
    Ok(())
}

fn main() {
    let cli = cli::Cli::parse();

    if cli.version {
        println!("zcat {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if cli.files.is_empty() {
        if let Err(e) = decompress_stdin() {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        for file in cli.files {
            if let Err(e) = decompress_file(&file) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}
