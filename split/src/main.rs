mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("split 0.1.0");
        std::process::exit(0);
    }

    let file = args.file.expect("file argument is required");

    if args.bytes.is_some() && args.lines.is_some() {
        eprintln!("split: cannot specify both -b and -l");
        std::process::exit(1);
    }

    if args.bytes.is_some() && args.number.is_some() {
        eprintln!("split: cannot specify both -b and -n");
        std::process::exit(1);
    }

    if args.lines.is_some() && args.number.is_some() {
        eprintln!("split: cannot specify both -l and -n");
        std::process::exit(1);
    }

    let result = if let Some(bytes_str) = &args.bytes {
        let bytes = parse_size(bytes_str).unwrap_or_else(|e| {
            eprintln!("split: {}", e);
            std::process::exit(1);
        });
        split_by_bytes(&file, bytes, &args.output, args.numeric_suffix, args.suffix_length)
    } else if let Some(lines) = args.lines {
        split_by_lines(&file, lines, &args.output, args.numeric_suffix, args.suffix_length)
    } else if let Some(num_pieces) = args.number {
        split_by_number(&file, num_pieces, &args.output, args.numeric_suffix, args.suffix_length)
    } else {
        split_by_lines(&file, 1000, &args.output, args.numeric_suffix, args.suffix_length)
    };

    if let Err(e) = result {
        eprintln!("split: {}", e);
        std::process::exit(1);
    }
}

fn parse_size(s: &str) -> Result<u64, String> {
    let suffix = s.chars().last().unwrap_or(' ');
    let (num_str, multiplier) = match suffix {
        'k' | 'K' => (&s[..s.len() - 1], 1024u64),
        'm' | 'M' => (&s[..s.len() - 1], 1024u64 * 1024),
        'g' | 'G' => (&s[..s.len() - 1], 1024u64 * 1024 * 1024),
        _ => (s, 1),
    };

    num_str.parse::<u64>()
        .map(|n| n * multiplier)
        .map_err(|e| format!("invalid size: {}", e))
}

fn generate_suffix(index: usize, numeric: bool, length: usize) -> String {
    if numeric {
        format!("{:0width$}", index, width = length)
    } else {
        let mut suffix = String::new();
        let mut n = index;
        let base = 26;

        loop {
            suffix.push((b'a' + (n % base) as u8) as char);
            n /= base;
            if n == 0 {
                break;
            }
        }

        let reversed: String = suffix.chars().rev().collect();
        format!("{:0>width$}", reversed, width = length)
    }
}

fn split_by_bytes(path: &Path, chunk_size: u64, prefix: &str, numeric: bool, suffix_len: usize) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let mut reader = BufReader::new(file);
    let mut buf = vec![0u8; chunk_size as usize];
    let mut index = 0;

    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("cannot read: {}", e))?;
        if n == 0 {
            break;
        }

        let suffix = generate_suffix(index, numeric, suffix_len);
        let output_name = format!("{}{}", prefix, suffix);

        let mut output_file = File::create(&output_name).map_err(|e| format!("cannot create file {}: {}", output_name, e))?;
        output_file.write_all(&buf[..n]).map_err(|e| format!("cannot write to {}: {}", output_name, e))?;

        index += 1;

        if n < chunk_size as usize {
            break;
        }
    }

    Ok(())
}

fn split_by_lines(path: &Path, lines_per_file: usize, prefix: &str, numeric: bool, suffix_len: usize) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut index = 0;
    let mut line_count = 0;
    let mut current_file: Option<File> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("cannot read line: {}", e))?;

        if line_count == 0 {
            let suffix = generate_suffix(index, numeric, suffix_len);
            let output_name = format!("{}{}", prefix, suffix);
            current_file = Some(File::create(&output_name).map_err(|e| format!("cannot create file {}: {}", output_name, e))?);
            index += 1;
        }

        let output = current_file.as_mut().unwrap();
        writeln!(output, "{}", line).map_err(|e| format!("cannot write to file: {}", e))?;

        line_count += 1;
        if line_count >= lines_per_file {
            line_count = 0;
            current_file = None;
        }
    }

    Ok(())
}

fn split_by_number(path: &Path, num_pieces: usize, prefix: &str, numeric: bool, suffix_len: usize) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let file_size = file.metadata().map_err(|e| format!("cannot get file metadata: {}", e))?.len();

    if num_pieces == 0 {
        return Err("cannot split into 0 pieces".to_string());
    }

    let chunk_size = (file_size + num_pieces as u64 - 1) / num_pieces as u64;
    split_by_bytes(path, chunk_size, prefix, numeric, suffix_len)
}