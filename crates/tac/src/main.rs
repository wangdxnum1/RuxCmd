mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("tac 0.1.0");
        return;
    }

    let sep: Vec<char> = if args.separator.is_empty() {
        vec!['\n']
    } else {
        args.separator.chars().collect()
    };

    // Each record carries its own separator (after the record body by
    // default, before it with -b). Records are emitted in reverse order.
    let mut records: Vec<Vec<char>> = Vec::new();
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = read_file_reader(stdin.lock(), &sep, args.before, &mut records) {
            eprintln!("tac: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = read_file_reader(stdin.lock(), &sep, args.before, &mut records) {
                    eprintln!("tac: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(p) {
                    Ok(f) => {
                        if let Err(e) = read_file_reader(f, &sep, args.before, &mut records) {
                            eprintln!("tac: {}: {}", p.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("tac: {}: {}", p.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    records.reverse();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for record in &records {
        let record_bytes: Vec<u8> = record
            .iter()
            .flat_map(|c| {
                let mut b = [0u8; 4];
                c.encode_utf8(&mut b).bytes().collect::<Vec<u8>>()
            })
            .collect();
        let _ = out.write_all(&record_bytes);
    }

    std::process::exit(exit_code);
}

/// Split input into records, with the separator attached to each record the
/// way GNU tac does:
///
/// - default: a record is `content + separator`; the final segment after the
///   last separator is a record without separator when non-empty (a trailing
///   separator produces no empty record).
/// - `before`: a record is `separator + content`; the leading segment before
///   the first separator is a record without separator when non-empty, and a
///   trailing separator forms a record of just the separator (empty content).
fn read_file_reader<R: Read>(
    mut reader: R,
    sep: &[char],
    before: bool,
    records: &mut Vec<Vec<char>>,
) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let chars: Vec<char> = buf.chars().collect();
    let sep_len = sep.len();

    // Non-overlapping separator occurrences, left to right.
    let mut sep_positions: Vec<usize> = Vec::new();
    let mut i = 0usize;
    while i + sep_len <= chars.len() {
        if chars[i..i + sep_len] == sep[..] {
            sep_positions.push(i);
            i += sep_len;
        } else {
            i += 1;
        }
    }

    if before {
        let first_sep = sep_positions.first().copied().unwrap_or(chars.len());
        if first_sep > 0 {
            records.push(chars[..first_sep].to_vec());
        }
        for (idx, &start) in sep_positions.iter().enumerate() {
            let end = sep_positions
                .get(idx + 1)
                .copied()
                .unwrap_or(chars.len());
            records.push(chars[start..end].to_vec());
        }
    } else {
        let mut prev_end = 0usize;
        for &start in &sep_positions {
            let mut record = chars[prev_end..start].to_vec();
            record.extend_from_slice(&chars[start..start + sep_len]);
            records.push(record);
            prev_end = start + sep_len;
        }
        if prev_end < chars.len() {
            records.push(chars[prev_end..].to_vec());
        }
    }
    Ok(())
}
