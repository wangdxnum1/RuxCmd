mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("shuf 0.1.0");
        return;
    }

    let seed: u64 = match args.random_source.as_ref() {
        Some(s) => hash_str_to_u64(s),
        None => {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);
            nanos ^ ((std::process::id() as u64) << 32)
        }
    };

    let mut items: Vec<String> = Vec::new();
    let mut exit_code = 0i32;

    if let Some(range) = args.input_range.as_ref() {
        let (lo, hi) = match parse_range(range) {
            Some(p) => p,
            None => {
                eprintln!("shuf: invalid range '{}'", range);
                std::process::exit(1);
            }
        };
        for n in lo..=hi {
            items.push(n.to_string());
        }
    } else {
        let mut read_lines = |r: Result<Box<dyn BufRead>, String>| {
            let reader = match r {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}", e);
                    exit_code = 1;
                    return;
                }
            };
            for line in reader.lines() {
                match line {
                    Ok(l) => items.push(l),
                    Err(e) => {
                        eprintln!("shuf: read error: {}", e);
                        exit_code = 1;
                    }
                }
            }
        };

        if args.files.is_empty() {
            let stdin = std::io::stdin();
            read_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
        } else {
            for p in &args.files {
                if p.to_string_lossy() == "-" {
                    let stdin = std::io::stdin();
                    read_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
                } else {
                    match File::open(p) {
                        Ok(f) => read_lines(Ok(Box::new(BufReader::new(f)))),
                        Err(e) => read_lines(Err(format!("shuf: {}: {}", p.display(), e))),
                    }
                }
            }
        }
    }

    shuffle(&mut items, seed);

    let n = std::cmp::min(args.head_count, items.len());
    let output: Vec<u8> = items
        .into_iter()
        .take(n)
        .flat_map(|l| {
            let mut v = l.into_bytes();
            v.push(b'\n');
            v
        })
        .collect();

    let wres: Result<(), std::io::Error> = match args.output.as_ref() {
        Some(p) => File::create(p).and_then(|mut f| f.write_all(&output)),
        None => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(&output)
        }
    };
    if let Err(e) = wres {
        eprintln!("shuf: write error: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

fn parse_range(s: &str) -> Option<(i64, i64)> {
    let dash = s.find('-')?;
    let lo: i64 = s[..dash].parse().ok()?;
    let hi: i64 = s[dash + 1..].parse().ok()?;
    if lo > hi { None } else { Some((lo, hi)) }
}

fn hash_str_to_u64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn shuffle<T>(v: &mut [T], seed: u64) {
    let mut s = seed;
    let n = v.len();
    for i in 0..n {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let range = (n - i) as u64;
        let r = (s >> 8) % range;
        let j = i + (r as usize);
        v.swap(i, j);
    }
}
