mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("expand 0.1.0");
        return;
    }

    let stops = parse_tabs(&args.tabs);
    if stops.is_err() {
        eprintln!("expand: invalid tabs value '{}'", args.tabs);
        std::process::exit(1);
    }
    let stops = stops.unwrap();

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process(BufReader::new(stdin.lock()), &stops, args.initial) {
            eprintln!("expand: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                process(BufReader::new(stdin.lock()), &stops, args.initial)
            } else {
                match File::open(p) {
                    Ok(f) => process(BufReader::new(f), &stops, args.initial),
                    Err(e) => {
                        eprintln!("expand: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("expand: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

enum Stops {
    Periodic(usize),
    Explicit(Vec<usize>),
}

fn parse_tabs(s: &str) -> Result<Stops, ()> {
    if let Ok(n) = s.parse::<usize>() {
        if n > 0 {
            return Ok(Stops::Periodic(n));
        }
    }
    let mut v: Vec<usize> = Vec::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()) {
        if part.is_empty() {
            continue;
        }
        match part.parse::<usize>() {
            Ok(x) if x > 0 => v.push(x),
            _ => return Err(()),
        }
    }
    if v.is_empty() {
        Err(())
    } else {
        v.sort();
        v.dedup();
        Ok(Stops::Explicit(v))
    }
}

fn next_stop(stops: &Stops, col: usize) -> usize {
    match stops {
        Stops::Periodic(n) => {
            if *n == 0 {
                col + 1
            } else {
                (col / n + 1) * n
            }
        }
        Stops::Explicit(v) => {
            for s in v {
                if *s > col {
                    return *s;
                }
            }
            col + 1
        }
    }
}

fn process<R: BufRead>(reader: R, stops: &Stops, initial: bool) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in reader.lines() {
        let line = line?;
        let mut col = 0usize;
        let mut past_init = false;
        for ch in line.chars() {
            match ch {
                '\t' if !past_init || !initial => {
                    let ns = next_stop(stops, col);
                    let n = if ns > col { ns - col } else { 1 };
                    for _ in 0..n {
                        out.write_all(b" ")?;
                    }
                    col = ns;
                }
                c => {
                    if c != ' ' && c != '\t' {
                        past_init = true;
                    }
                    let mut b = [0u8; 4];
                    let s = c.encode_utf8(&mut b);
                    out.write_all(s.as_bytes())?;
                    col += 1;
                }
            }
        }
        out.write_all(b"\n")?;
    }
    Ok(())
}
