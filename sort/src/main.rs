mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    let mut lines = Vec::new();

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        read_lines(stdin.lock(), &mut lines);
    } else {
        for file in &args.files {
            let file_name = file.display().to_string();
            match fs::File::open(file) {
                Ok(f) => read_lines(BufReader::new(f), &mut lines),
                Err(e) => eprintln!("sort: {}: {}", file_name, e),
            }
        }
    }

    if args.check {
        if is_sorted(&lines, &args) {
            std::process::exit(0);
        } else {
            eprintln!("sort: disorder detected");
            std::process::exit(1);
        }
    }

    if args.unique {
        let mut unique_lines = Vec::new();
        for line in lines {
            if unique_lines.is_empty() || unique_lines.last() != Some(&line) {
                unique_lines.push(line);
            }
        }
        lines = unique_lines;
    }

    lines.sort_by(|a, b| compare_lines(a, b, &args));

    if args.reverse {
        lines.reverse();
    }

    for line in lines {
        println!("{}", line);
    }
}

fn read_lines<R: BufRead>(mut reader: R, lines: &mut Vec<String>) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap_or(0) > 0 {
        lines.push(buf.trim_end().to_string());
        buf.clear();
    }
}

fn is_sorted(lines: &[String], args: &cli::Args) -> bool {
    for i in 1..lines.len() {
        let cmp = compare_lines(&lines[i - 1], &lines[i], args);
        if args.reverse {
            if cmp > std::cmp::Ordering::Equal {
                return false;
            }
        } else {
            if cmp > std::cmp::Ordering::Equal {
                return false;
            }
        }
    }
    true
}

fn compare_lines(a: &str, b: &str, args: &cli::Args) -> std::cmp::Ordering {
    let a_val = if args.ignore_case { a.to_lowercase() } else { a.to_string() };
    let b_val = if args.ignore_case { b.to_lowercase() } else { b.to_string() };

    if args.numeric {
        match (parse_number(&a_val), parse_number(&b_val)) {
            (Some(na), Some(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
            _ => a_val.cmp(&b_val),
        }
    } else if args.human_numeric {
        match (parse_human_number(&a_val), parse_human_number(&b_val)) {
            (Some(na), Some(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
            _ => a_val.cmp(&b_val),
        }
    } else {
        a_val.cmp(&b_val)
    }
}

fn parse_number(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

fn parse_human_number(s: &str) -> Option<f64> {
    let s = s.trim().to_lowercase();
    let suffixes = [("k", 1000.0), ("m", 1_000_000.0), ("g", 1_000_000_000.0), ("t", 1_000_000_000_000.0)];
    
    for (suffix, multiplier) in suffixes.iter() {
        if s.ends_with(suffix) {
            if let Ok(num) = s[..s.len() - suffix.len()].trim().parse::<f64>() {
                return Some(num * multiplier);
            }
        }
    }
    
    s.parse().ok()
}