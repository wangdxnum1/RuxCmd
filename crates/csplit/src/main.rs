mod cli;

use clap::Parser;
use std::fs::{File, remove_file};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("csplit 0.1.0");
        return;
    }

    if args.patterns.is_empty() {
        eprintln!("csplit: missing pattern(s)");
        std::process::exit(1);
    }

    let lines: Vec<String> = match read_input(&args.file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("csplit: {}", e);
            std::process::exit(1);
        }
    };

    let mut splits: Vec<(usize, usize)> = Vec::new();
    let mut cur: usize = 0;
    let mut created: Vec<PathBuf> = Vec::new();
    let mut exit_code = 0i32;

    let mut pat_idx: usize = 0;
    while pat_idx < args.patterns.len() {
        let p = &args.patterns[pat_idx];
        if p.starts_with('/') && p.ends_with('/') && p.len() >= 2 {
            let pat = &p[1..p.len() - 1];
            match find_pattern(pat, &lines, cur) {
                Some(lineno) => {
                    // The matched line starts the next output file, so it is
                    // never dropped from the output.
                    splits.push((cur, lineno));
                    cur = lineno;
                }
                None => {
                    eprintln!("csplit: '{}': match not found", p);
                    exit_code = 1;
                    // Flush the remaining input, then stop: GNU csplit aborts
                    // on an unmatched pattern.
                    splits.push((cur, lines.len()));
                    cur = lines.len();
                    break;
                }
            }
            pat_idx += 1;
        } else if let Ok(n) = p.parse::<usize>() {
            if n == 0 || n > lines.len() {
                splits.push((cur, lines.len()));
                cur = lines.len();
            } else {
                splits.push((cur, n - 1));
                cur = n - 1;
            }
            pat_idx += 1;
        } else {
            // Only `/substring/` and line numbers are supported. In
            // particular GNU offsets (`/re/+N`, `/re/-N`) and repeats
            // (`/re/{N}`, the homemade `/re/N`) are invalid here and must
            // fail loudly instead of being silently ignored.
            eprintln!("csplit: invalid pattern '{}'", p);
            exit_code = 1;
            break;
        }
    }
    if cur < lines.len() {
        splits.push((cur, lines.len()));
    }

    let mut file_count = 0usize;
    for (s, e) in &splits {
        if args.elide_empty && s >= e {
            continue;
        }
        let fname = format_filename(&args.prefix, file_count, args.digits);
        let path = PathBuf::from(&fname);
        match File::create(&path) {
            Ok(mut f) => {
                for i in *s..(*e.min(&lines.len())) {
                    let _ = writeln!(f, "{}", lines[i]);
                }
                let bytes: usize = lines[*s..(*e.min(&lines.len()))]
                    .iter()
                    .map(|l| l.len() + 1)
                    .sum();
                if !args.quiet {
                    println!("{}", bytes);
                }
                created.push(path);
                file_count += 1;
            }
            Err(err) => {
                eprintln!("csplit: {}: {}", fname, err);
                exit_code = 1;
            }
        }
    }

    if exit_code != 0 && !args.keep {
        for p in &created {
            let _ = remove_file(p);
        }
    }

    std::process::exit(exit_code);
}

fn read_input(p: &Option<PathBuf>) -> Result<Vec<String>, String> {
    let mut reader: Box<dyn BufRead> = match p {
        Some(pp) if pp.to_string_lossy() != "-" => {
            let f = File::open(pp).map_err(|e| format!("{}: {}", pp.display(), e))?;
            Box::new(BufReader::new(f))
        }
        _ => {
            let stdin = std::io::stdin();
            Box::new(BufReader::new(stdin.lock()))
        }
    };
    let mut out = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => out.push(l),
            Err(e) => return Err(format!("read error: {}", e)),
        }
    }
    Ok(out)
}

fn find_pattern(pat: &str, lines: &[String], start: usize) -> Option<usize> {
    for (i, l) in lines.iter().enumerate().skip(start) {
        if l.contains(pat) {
            return Some(i);
        }
    }
    None
}

fn format_filename(prefix: &str, idx: usize, digits: usize) -> String {
    format!("{}{:0w$}", prefix, idx, w = digits)
}
