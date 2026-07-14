mod cli;

use clap::Parser;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("factor 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.numbers.is_empty() {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let trimmed = l.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Err(e) = factor_one(trimmed) {
                        eprintln!("factor: {}", e);
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("factor: error reading stdin: {}", e);
                    exit_code = 1;
                    break;
                }
            }
        }
    } else {
        for n in &args.numbers {
            if let Err(e) = factor_one(n) {
                eprintln!("factor: {}", e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

fn factor_one(raw: &str) -> Result<(), String> {
    let n: u64 = raw
        .parse()
        .map_err(|_| format!("'{}' is not a valid positive integer", raw))?;
    if n == 0 {
        return Err("'0' is not a valid positive integer".to_string());
    }
    if n == 1 {
        println!("1:");
        return Ok(());
    }
    let mut factors: Vec<u64> = Vec::new();
    let mut x = n;
    while x % 2 == 0 {
        factors.push(2);
        x /= 2;
    }
    let mut d: u64 = 3;
    while d.saturating_mul(d) <= x {
        while x % d == 0 {
            factors.push(d);
            x /= d;
        }
        d = d.saturating_add(2);
    }
    if x > 1 {
        factors.push(x);
    }
    let out: Vec<String> = factors.iter().map(|f| f.to_string()).collect();
    println!("{}: {}", n, out.join(" "));
    Ok(())
}
