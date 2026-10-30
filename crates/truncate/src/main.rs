use clap::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Shrink or extend the size of each file - a Windows port of Linux truncate.
#[derive(Parser, Debug)]
#[command(name = "truncate", version, about)]
pub struct Args {
    /// Do not create files that do not exist
    #[arg(short = 'c', long = "no-create")]
    pub no_create: bool,

    /// Set or adjust the file size by SIZE bytes
    #[arg(
        short = 's',
        long = "size",
        value_name = "SIZE",
        allow_hyphen_values = true
    )]
    pub size: Option<String>,

    /// Base the size on that of RFILE
    #[arg(short = 'r', long = "reference", value_name = "RFILE")]
    pub reference: Option<PathBuf>,

    /// Files to adjust
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    if args.files.is_empty() {
        eprintln!("truncate: missing operand");
        std::process::exit(1);
    }

    let target: (i128, bool) = if let Some(reference) = &args.reference {
        match std::fs::metadata(reference) {
            Ok(m) => (m.len() as i128, false),
            Err(e) => {
                eprintln!(
                    "truncate: failed to get attributes of '{}': {}",
                    reference.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    } else {
        match &args.size {
            Some(spec) => match parse_size(spec) {
                Some(pair) => pair,
                None => {
                    eprintln!("truncate: Invalid number: '{}'", spec);
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("truncate: You must specify -s or -r");
                std::process::exit(1);
            }
        }
    };
    let (amount, at_least) = target;

    let mut failed = false;
    for file in &args.files {
        if !file.exists() && args.no_create {
            failed = true;
            continue;
        }
        let mut open = OpenOptions::new();
        open.write(true).create(true);
        match open.open(file) {
            Ok(mut handle) => {
                let current = handle.metadata().map(|m| m.len() as i128).unwrap_or(0);
                let new_size: i128 = if at_least {
                    current.max(amount)
                } else if amount < 0 {
                    (current + amount).max(0)
                } else {
                    amount
                };
                if let Err(e) = handle.set_len(new_size.max(0) as u64) {
                    eprintln!("truncate: cannot resize '{}': {}", file.display(), e);
                    failed = true;
                }
                let _ = handle.flush();
            }
            Err(e) => {
                eprintln!("truncate: cannot open '{}': {}", file.display(), e);
                failed = true;
            }
        }
    }
    std::process::exit(if failed { 1 } else { 0 });
}

/// Parse a GNU truncate size: optional +/- (with + meaning "at least"),
/// digits, and an optional K/M/G/T/P/E (1024-based) or KB/MB/... (1000-based)
/// suffix. Returns (bytes, at_least).
fn parse_size(spec: &str) -> Option<(i128, bool)> {
    let mut at_least = false;
    let mut body = spec;
    if let Some(rest) = body.strip_prefix('+') {
        at_least = true;
        body = rest;
    }
    let split = body.find(|c: char| c.is_ascii_alphabetic());
    let (num_part, suffix) = match split {
        Some(idx) => (&body[..idx], body[idx..].to_ascii_uppercase()),
        None => (body, String::new()),
    };
    let num: i128 = num_part.parse().ok()?;
    let multiplier: i128 = match suffix.as_str() {
        "" => 1,
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        "T" => 1024i128 * 1024 * 1024 * 1024,
        "P" => 1024i128 * 1024 * 1024 * 1024 * 1024,
        "E" => 1024i128 * 1024 * 1024 * 1024 * 1024 * 1024,
        "KB" => 1000,
        "MB" => 1000 * 1000,
        "GB" => 1000 * 1000 * 1000,
        _ => return None,
    };
    Some((num.checked_mul(multiplier)?, at_least))
}
