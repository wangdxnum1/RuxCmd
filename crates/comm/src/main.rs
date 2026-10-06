mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("comm 0.1.0");
        process::exit(0);
    }

    let file1 = match &cli.file1 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE1>");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: comm [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let file2 = match &cli.file2 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: comm [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let f1 = match File::open(file1) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("comm: cannot open '{}': {}", file1, e);
            process::exit(1);
        }
    };

    let f2 = match File::open(file2) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("comm: cannot open '{}': {}", file2, e);
            process::exit(1);
        }
    };

    let mut reader1 = BufReader::new(f1).lines();
    let mut reader2 = BufReader::new(f2).lines();

    let mut line1 = reader1.next();
    let mut line2 = reader2.next();

    loop {
        match (line1.as_ref(), line2.as_ref()) {
            (Some(Ok(l1)), Some(Ok(l2))) => match l1.cmp(l2) {
                std::cmp::Ordering::Less => {
                    if !cli.hide_col1 {
                        println!("{}", l1);
                    }
                    line1 = reader1.next();
                }
                std::cmp::Ordering::Greater => {
                    if !cli.hide_col2 {
                        println!("{}\t{}", "", l2);
                    }
                    line2 = reader2.next();
                }
                std::cmp::Ordering::Equal => {
                    if !cli.hide_col3 {
                        println!("{}\t{}\t{}", "", "", l1);
                    }
                    line1 = reader1.next();
                    line2 = reader2.next();
                }
            },
            (Some(Ok(l1)), None) => {
                if !cli.hide_col1 {
                    println!("{}", l1);
                }
                line1 = reader1.next();
            }
            (None, Some(Ok(l2))) => {
                if !cli.hide_col2 {
                    println!("{}\t{}", "", l2);
                }
                line2 = reader2.next();
            }
            (Some(Err(e)), _) => {
                eprintln!("comm: error reading '{}': {}", file1, e);
                process::exit(1);
            }
            (_, Some(Err(e))) => {
                eprintln!("comm: error reading '{}': {}", file2, e);
                process::exit(1);
            }
            (None, None) => {
                process::exit(0);
            }
        }
    }
}
