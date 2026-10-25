mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() {
    let args = cli::Cli::parse();
    if args.version {
        println!("uniq 0.1.0");
        return;
    }
    let result = run(&args);
    if let Err(error) = result {
        eprintln!("uniq: {error}");
        std::process::exit(1);
    }
}

fn run(args: &cli::Cli) -> io::Result<()> {
    let reader: Box<dyn BufRead> = match args.file.as_deref() {
        Some(path) if path != "-" => Box::new(BufReader::new(File::open(path)?)),
        _ => Box::new(BufReader::new(io::stdin())),
    };
    let separator = if args.zero_terminated { 0 } else { b'\n' };
    let mut previous: Option<(Vec<u8>, Vec<u8>, usize)> = None;
    let mut out = io::BufWriter::new(io::stdout().lock());
    for record in ByteLines::with_separator(reader, separator) {
        let record = record?;
        let key = make_key(&record, args);
        match previous.as_mut() {
            Some((_, old_key, count)) if *old_key == key => *count += 1,
            _ => {
                if let Some((record, _, count)) = previous.take() {
                    print_record(&mut out, &record, count, args, separator)?;
                }
                previous = Some((record, key, 1));
            }
        }
    }
    if let Some((record, _, count)) = previous {
        print_record(&mut out, &record, count, args, separator)?;
    }
    out.flush()
}

fn make_key(record: &[u8], args: &cli::Cli) -> Vec<u8> {
    let mut position = 0;
    for _ in 0..args.skip_fields.unwrap_or(0) {
        while position < record.len() && matches!(record[position], b' ' | b'\t' | b'\n') {
            position += 1;
        }
        while position < record.len() && !matches!(record[position], b' ' | b'\t' | b'\n') {
            position += 1;
        }
    }
    let key = &record[position..];
    if args.ignore_case {
        match std::str::from_utf8(key) {
            Ok(text) => text.to_lowercase().into_bytes(),
            Err(_) => key.iter().map(u8::to_ascii_lowercase).collect(),
        }
    } else {
        key.to_vec()
    }
}

fn print_record<W: Write>(
    out: &mut W,
    record: &[u8],
    count: usize,
    args: &cli::Cli,
    separator: u8,
) -> io::Result<()> {
    if (args.duplicate_only && count == 1) || (args.unique_only && count > 1) {
        return Ok(());
    }
    if args.count {
        write!(out, "{:>7} ", count)?;
    }
    out.write_all(record)?;
    out.write_all(&[separator])
}
