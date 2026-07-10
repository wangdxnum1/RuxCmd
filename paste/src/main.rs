mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    let delimiter = args.delimiters.unwrap_or_else(|| "\t".to_string());
    let serial = args.serial;

    let mut exit_code = 0i32;

    let readers: Vec<Box<dyn BufRead>> = if args.files.is_empty() {
        vec![Box::new(BufReader::new(std::io::stdin()))]
    } else {
        args.files
            .iter()
            .map(|path| {
                let path_str = path.to_string_lossy();
                if path_str == "-" {
                    Box::new(BufReader::new(std::io::stdin())) as Box<dyn BufRead>
                } else {
                    match File::open(path) {
                        Ok(file) => Box::new(BufReader::new(file)) as Box<dyn BufRead>,
                        Err(e) => {
                            eprintln!("paste: {}: {}", path.display(), e);
                            exit_code = 1;
                            Box::new(BufReader::new(std::io::empty())) as Box<dyn BufRead>
                        }
                    }
                }
            })
            .collect()
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    if serial {
        if let Err(e) = paste_serial(readers, &delimiter) {
            eprintln!("paste: {}", e);
            exit_code = 1;
        }
    } else {
        if let Err(e) = paste_parallel(readers, &delimiter) {
            eprintln!("paste: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn paste_parallel(mut readers: Vec<Box<dyn BufRead>>, delimiter: &str) -> Result<(), std::io::Error> {
    let mut lines: Vec<Option<String>> = readers.iter_mut().map(|_| None).collect();
    let mut eof_flags: Vec<bool> = readers.iter().map(|_| false).collect();
    let mut active_readers = readers.len();

    loop {
        for (i, reader) in readers.iter_mut().enumerate() {
            if !eof_flags[i] && lines[i].is_none() {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        eof_flags[i] = true;
                        active_readers -= 1;
                    }
                    Ok(_) => {
                        lines[i] = Some(line.trim_end_matches('\n').to_string());
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        if active_readers == 0 {
            break;
        }

        let output: Vec<String> = lines
            .iter()
            .map(|line| line.as_deref().unwrap_or("").to_string())
            .collect();

        println!("{}", output.join(delimiter));

        for line in lines.iter_mut() {
            *line = None;
        }
    }

    Ok(())
}

fn paste_serial(mut readers: Vec<Box<dyn BufRead>>, delimiter: &str) -> Result<(), std::io::Error> {
    for reader in readers.iter_mut() {
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();

        loop {
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    lines.push(line.trim_end_matches('\n').to_string());
                    line.clear();
                }
                Err(e) => return Err(e),
            }
        }

        if !lines.is_empty() {
            println!("{}", lines.join(delimiter));
        }
    }

    Ok(())
}