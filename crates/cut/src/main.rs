mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();
    let delimiter = args.delimiter.unwrap_or('\t');
    let output_delimiter = args
        .output_delimiter
        .clone()
        .unwrap_or_else(|| delimiter.to_string());
    let separator = if args.zero_terminated { 0 } else { b'\n' };
    let files = if args.files.is_empty() {
        vec![std::path::PathBuf::from("-")]
    } else {
        args.files.clone()
    };
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut failed = false;
    for path in files {
        let reader: io::Result<Box<dyn BufRead>> = if path.as_os_str() == "-" {
            Ok(Box::new(BufReader::new(io::stdin())))
        } else {
            File::open(&path).map(|file| Box::new(BufReader::new(file)) as Box<dyn BufRead>)
        };
        let result = reader.and_then(|reader| {
            process_records(
                reader,
                &args,
                delimiter,
                &output_delimiter,
                separator,
                &mut out,
            )
        });
        if let Err(error) = result {
            eprintln!("cut: {}: {}", path.display(), error);
            failed = true;
        }
    }
    if let Err(error) = out.flush() {
        eprintln!("cut: {error}");
        failed = true;
    }
    if failed {
        std::process::exit(1);
    }
}

fn process_records<R: BufRead, W: Write>(
    reader: R,
    args: &cli::Args,
    delimiter: char,
    output_delimiter: &str,
    separator: u8,
    out: &mut W,
) -> io::Result<()> {
    let mut encoded = [0; 4];
    let delimiter_bytes = delimiter.encode_utf8(&mut encoded).as_bytes();
    for record in ByteLines::with_separator(reader, separator) {
        let raw = record?;
        let has_delimiter = raw
            .windows(delimiter_bytes.len())
            .any(|part| part == delimiter_bytes);
        // Preserve this port's existing -s handling for byte/character modes.
        if args.only_delimited && !has_delimiter {
            continue;
        }
        let result = if let Some(spec) = &args.bytes {
            parse_spec(spec, raw.len())
                .into_iter()
                .filter_map(|index| raw.get(index).copied())
                .collect()
        } else if let Some(spec) = &args.fields {
            cut_fields(&raw, spec, delimiter_bytes, output_delimiter.as_bytes())
        } else if let Some(spec) = &args.characters {
            cut_chars(&raw, spec)
        } else {
            raw
        };
        out.write_all(&result)?;
        out.write_all(&[separator])?;
    }
    Ok(())
}

fn cut_fields(line: &[u8], spec: &str, delimiter: &[u8], output_delimiter: &[u8]) -> Vec<u8> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut position = 0;
    while position + delimiter.len() <= line.len() {
        if &line[position..position + delimiter.len()] == delimiter {
            fields.push(&line[start..position]);
            position += delimiter.len();
            start = position;
        } else {
            position += 1;
        }
    }
    if fields.is_empty() {
        return line.to_vec();
    }
    fields.push(&line[start..]);
    let mut result = Vec::new();
    let mut first = true;
    for index in parse_spec(spec, fields.len()) {
        if let Some(field) = fields.get(index) {
            if !first {
                result.extend_from_slice(output_delimiter);
            }
            result.extend_from_slice(field);
            first = false;
        }
    }
    result
}

fn cut_chars(raw: &[u8], spec: &str) -> Vec<u8> {
    match std::str::from_utf8(raw) {
        Ok(text) => {
            let mut starts: Vec<_> = text.char_indices().map(|(index, _)| index).collect();
            starts.push(raw.len());
            let mut result = Vec::new();
            for index in parse_spec(spec, starts.len() - 1) {
                if index + 1 < starts.len() {
                    result.extend_from_slice(&raw[starts[index]..starts[index + 1]]);
                }
            }
            result
        }
        Err(_) => parse_spec(spec, raw.len())
            .into_iter()
            .filter_map(|index| raw.get(index).copied())
            .collect(),
    }
}

fn parse_spec(spec: &str, max_len: usize) -> Vec<usize> {
    let mut indices = Vec::new();
    for part in spec.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            let start = range[0].parse::<usize>().unwrap_or(1).saturating_sub(1);
            let end = if range.len() > 1 && !range[1].is_empty() {
                range[1].parse::<usize>().unwrap_or(max_len)
            } else {
                max_len
            };
            indices.extend(start..end.min(max_len));
        } else {
            let index = part.parse::<usize>().unwrap_or(1).saturating_sub(1);
            if index < max_len {
                indices.push(index);
            }
        }
    }
    indices
}
