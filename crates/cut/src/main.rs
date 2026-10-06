mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    let delimiter = args.delimiter.unwrap_or('\t');
    let output_delimiter = args
        .output_delimiter
        .clone()
        .unwrap_or_else(|| delimiter.to_string());

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        process_lines(stdin.lock(), "<stdin>", &args, delimiter, &output_delimiter);
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
                ),
                Err(e) => eprintln!("cut: {}: {}", file_name, e),
            }
        }
    }
}

fn process_lines<R: BufRead>(
    mut reader: R,
    _name: &str,
    args: &cli::Args,
    delimiter: char,
    output_delimiter: &str,
) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap_or(0) > 0 {
        let line = buf.trim_end();

        if args.only_delimited && !line.contains(delimiter) {
            buf.clear();
            continue;
        }

        let result = if let Some(fields_spec) = &args.fields {
            cut_fields(line, fields_spec, delimiter, output_delimiter)
        } else if let Some(bytes_spec) = &args.bytes {
            cut_bytes(line, bytes_spec)
        } else if let Some(chars_spec) = &args.characters {
            cut_chars(line, chars_spec)
        } else {
            line.to_string()
        };

        println!("{}", result);
        buf.clear();
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

fn cut_bytes(line: &str, spec: &str) -> String {
    let bytes = line.as_bytes();
    let indices = parse_spec(spec, bytes.len());

    indices
        .iter()
        .filter(|&&i| i < bytes.len())
        .map(|&i| bytes[i] as char)
        .collect()
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
