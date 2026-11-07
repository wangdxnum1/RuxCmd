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

    /// Set or adjust the file size by SIZE bytes; SIZE may be prefixed by an
    /// adjusting character: '+' extend by, '-' reduce by, '<' at most,
    /// '>' at least, '/' round down to multiple of, '%' round up to multiple of
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

    /// Treat SIZE as a count of 4096-byte IO blocks instead of bytes
    #[arg(short = 'o', long = "io-blocks")]
    pub io_blocks: bool,

    /// Files to adjust
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// How SIZE relates to the current (or reference) file size, matching the
/// GNU adjusting characters.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Adjust {
    Set,
    Add,
    Sub,
    AtMost,
    AtLeast,
    RoundDown,
    RoundUp,
}

fn main() {
    let args = Args::parse();
    if args.files.is_empty() {
        eprintln!("truncate: missing operand");
        std::process::exit(1);
    }
    if args.io_blocks && args.size.is_none() {
        eprintln!("truncate: '--io-blocks' was specified but '--size' was not");
        std::process::exit(1);
    }

    let spec = args
        .size
        .as_deref()
        .map(|spec| match parse_size(spec, args.io_blocks) {
            Some((adjust, amount)) => {
                if amount == 0 && matches!(adjust, Adjust::RoundDown | Adjust::RoundUp) {
                    eprintln!("truncate: division by zero");
                    std::process::exit(1);
                }
                (adjust, amount)
            }
            None => {
                eprintln!("truncate: Invalid number: '{spec}'");
                std::process::exit(1);
            }
        });

    // With --reference the final size comes from RFILE, optionally adjusted
    // by a relative +N/-N size; GNU rejects absolute sizes there.
    let reference_target: Option<i128> = if let Some(reference) = &args.reference {
        let ref_size = match std::fs::metadata(reference) {
            Ok(m) => m.len() as i128,
            Err(e) => {
                eprintln!(
                    "truncate: failed to get attributes of '{}': {}",
                    reference.display(),
                    e
                );
                std::process::exit(1);
            }
        };
        Some(match spec {
            None => ref_size,
            Some((Adjust::Add, amount)) => (ref_size + amount).max(0),
            Some((Adjust::Sub, amount)) => (ref_size - amount).max(0),
            Some(_) => {
                eprintln!("truncate: you must specify a relative '--size' with '--reference'");
                std::process::exit(1);
            }
        })
    } else if spec.is_none() {
        eprintln!("truncate: You must specify -s or -r");
        std::process::exit(1);
    } else {
        None
    };

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
                let new_size = match reference_target {
                    Some(target) => target,
                    None => {
                        let (adjust, amount) = spec.expect("checked above");
                        let current = handle.metadata().map(|m| m.len() as i128).unwrap_or(0);
                        apply_adjust(current, adjust, amount)
                    }
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

/// Apply one GNU adjusting operation to the current size.  Results that
/// would go below zero clamp to zero like GNU (e.g. `-s -1B` on a tiny file).
fn apply_adjust(current: i128, adjust: Adjust, amount: i128) -> i128 {
    match adjust {
        Adjust::Set => amount,
        Adjust::Add => (current + amount).max(0),
        Adjust::Sub => (current - amount).max(0),
        Adjust::AtMost => current.min(amount).max(0),
        Adjust::AtLeast => current.max(amount),
        Adjust::RoundDown => amount * (current / amount),
        Adjust::RoundUp => amount * (current / amount + i128::from(current % amount != 0)),
    }
}

/// Parse a GNU truncate SIZE: optional adjusting character, digits, and an
/// optional K/M/G/T/P/E (1024-based) or KB/MB/... (1000-based) suffix.  With
/// `io_blocks` the byte total is multiplied by the 4096-byte IO block size.
/// Returns None for malformed numbers.
fn parse_size(spec: &str, io_blocks: bool) -> Option<(Adjust, i128)> {
    let mut body = spec;
    let adjust = match body.chars().next() {
        Some('+') => {
            body = &body[1..];
            Adjust::Add
        }
        Some('-') => {
            body = &body[1..];
            Adjust::Sub
        }
        Some('<') => {
            body = &body[1..];
            Adjust::AtMost
        }
        Some('>') => {
            body = &body[1..];
            Adjust::AtLeast
        }
        Some('/') => {
            body = &body[1..];
            Adjust::RoundDown
        }
        Some('%') => {
            body = &body[1..];
            Adjust::RoundUp
        }
        _ => Adjust::Set,
    };
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
    let mut amount = num.checked_mul(multiplier)?;
    if io_blocks {
        amount = amount.checked_mul(4096)?;
    }
    if amount < 0 && matches!(adjust, Adjust::RoundDown | Adjust::RoundUp) {
        return None;
    }
    Some((adjust, amount))
}
