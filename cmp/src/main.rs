mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("cmp 0.1.0");
        process::exit(0);
    }

    let file1 = match &cli.file1 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE1>");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: cmp [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let file2 = match &cli.file2 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: cmp [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let f1 = match File::open(file1) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cmp: cannot open '{}': {}", file1, e);
            process::exit(1);
        }
    };

    let f2 = match File::open(file2) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cmp: cannot open '{}': {}", file2, e);
            process::exit(1);
        }
    };

    let mut reader1 = BufReader::new(f1);
    let mut reader2 = BufReader::new(f2);

    let mut byte1 = [0u8];
    let mut byte2 = [0u8];
    let mut position = 1u64;

    loop {
        let bytes_read1 = reader1.read(&mut byte1).unwrap_or_else(|e| {
            eprintln!("cmp: error reading '{}': {}", file1, e);
            process::exit(1);
        });

        let bytes_read2 = reader2.read(&mut byte2).unwrap_or_else(|e| {
            eprintln!("cmp: error reading '{}': {}", file2, e);
            process::exit(1);
        });

        if bytes_read1 == 0 && bytes_read2 == 0 {
            process::exit(0);
        }

        if bytes_read1 == 0 || bytes_read2 == 0 {
            if !cli.silent {
                if bytes_read1 == 0 {
                    eprintln!("cmp: EOF on {}", file1);
                } else {
                    eprintln!("cmp: EOF on {}", file2);
                }
            }
            process::exit(1);
        }

        if byte1[0] != byte2[0] {
            if cli.verbose {
                println!("{} {} {}", position, byte1[0], byte2[0]);
            } else if !cli.silent {
                eprintln!("cmp: {} {} differ: byte {}, line {}", file1, file2, position, position);
            }
            process::exit(1);
        }

        position += 1;
    }
}