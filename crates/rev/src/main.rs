mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("rev 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = BufReader::new(std::io::stdin());
        if let Err(e) = process_reader(stdin) {
            eprintln!("rev: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = BufReader::new(std::io::stdin());
                if let Err(e) = process_reader(stdin) {
                    eprintln!("rev: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_reader(reader) {
                            eprintln!("rev: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("rev: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_reader<R: BufRead>(mut reader: R) -> Result<(), std::io::Error> {
    let mut line = String::new();
    loop {
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let mut reversed = line.chars().rev().collect::<String>();

        let has_newline = line.ends_with('\n');
        if has_newline {
            reversed = reversed.replacen('\n', "", 1);
            reversed.push('\n');
        }

        print!("{}", reversed);
        line.clear();
    }
    Ok(())
}
