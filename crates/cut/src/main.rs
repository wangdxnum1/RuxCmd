mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();

    let delimiter = args.delimiter.unwrap_or('\t');
    let output_delimiter = args
        .output_delimiter
        .clone()
        .unwrap_or_else(|| delimiter.to_string());

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        process_lines(
            stdin.lock(),
            "<stdin>",
            &args,
            delimiter,
            &output_delimiter,
            &mut out,
        );
    } else {
        for file in &args.files {
            let file_name = file.display().to_string();
            match fs::File::open(file) {
                Ok(f) => process_lines(
                    BufReader::new(f),
                    &file_name,
                    &args,
                    delimiter,
                    &output_delimiter,
                    &mut out,
                ),
                Err(e) => eprintln!("cut: {}: {}", file_name, e),
            }
        }
    }

    let _ = out.flush();
}

/// Read lines as raw bytes: field/character selection matches on lossy UTF-8,
/// `-b` and the no-selection passthrough echo the original bytes. A read error
/// is reported on stderr instead of silently truncating the rest of the input.
fn process_lines<R: BufRead>(
    reader: R,
    name: &str,
    args: &cli::Args,
    delimiter: char,
    output_delimiter: &str,
    out: &mut dyn Write,
) {
    for entry in ByteLines::new(reader) {
        let raw = match entry {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("cut: {}: read error: {}", name, e);
                break;
            }
        };

        let line = String::from_utf8_lossy(&raw);

        // P2 (unchanged this round): -s also applies to -b/-c modes.
        if args.only_delimited && !line.contains(delimiter) {
            continue;
        }

        if let Some(bytes_spec) = &args.bytes {
            let indices = parse_spec(bytes_spec, raw.len());
            for &i in &indices {
                if i < raw.len() {
                    let _ = out.write_all(&raw[i..i + 1]);
                }
            }
            let _ = out.write_all(b"\n");
            continue;
        }

        let result = if let Some(fields_spec) = &args.fields {
            cut_fields(&line, fields_spec, delimiter, output_delimiter)
        } else if let Some(chars_spec) = &args.characters {
            cut_chars(&line, chars_spec)
        } else {
            // No selection mode: pass the original line through byte for byte.
            let _ = out.write_all(&raw);
            let _ = out.write_all(b"\n");
            continue;
        };

        let _ = writeln!(out, "{}", result);
    }
}

fn cut_fields(line: &str, spec: &str, delimiter: char, output_delimiter: &str) -> String {
    let fields: Vec<&str> = line.split(delimiter).collect();
    let indices = parse_spec(spec, fields.len());

    indices
        .iter()
        .filter(|&&i| i < fields.len())
        .map(|&i| fields[i])
        .collect::<Vec<&str>>()
        .join(output_delimiter)
}

fn cut_chars(line: &str, spec: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let indices = parse_spec(spec, chars.len());

    indices
        .iter()
        .filter(|&&i| i < chars.len())
        .map(|&i| chars[i])
        .collect()
}

fn parse_spec(spec: &str, max_len: usize) -> Vec<usize> {
    let mut indices = Vec::new();

    for part in spec.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            let start: usize = range[0].parse::<usize>().unwrap_or(1).saturating_sub(1);
            let end: usize = if range.len() > 1 && !range[1].is_empty() {
                range[1].parse::<usize>().unwrap_or(max_len)
            } else {
                max_len
            };

            for i in start..end {
                indices.push(i);
            }
        } else {
            let idx: usize = part.parse::<usize>().unwrap_or(1).saturating_sub(1);
            if idx < max_len {
                indices.push(idx);
            }
        }
    }

    indices
}
