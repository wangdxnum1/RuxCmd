mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

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

    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = read_file_reader(stdin.lock(), &sep, &mut lines) {
            eprintln!("tac: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = read_file_reader(stdin.lock(), &sep, &mut lines) {
                    eprintln!("tac: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(p) {
                    Ok(f) => {
                        if let Err(e) = read_file_reader(f, &sep, &mut lines) {
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

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let sep_bytes: Vec<u8> = sep
        .iter()
        .flat_map(|c| {
            let mut b = [0u8; 4];
            c.encode_utf8(&mut b).bytes().collect::<Vec<u8>>()
        })
        .collect();

    lines.reverse();

    for line in &lines {
        let line_bytes: Vec<u8> = line
            .iter()
            .flat_map(|c| {
                let mut b = [0u8; 4];
                c.encode_utf8(&mut b).bytes().collect::<Vec<u8>>()
            })
            .collect();

        if args.before {
            // separator is BEFORE current line's body
            let _ = out.write_all(&sep_bytes);
            let _ = out.write_all(&line_bytes);
        } else {
            // separator is AFTER current line's body (FIXED: was duplicate BEFORE)
            let _ = out.write_all(&line_bytes);
            let _ = out.write_all(&sep_bytes);
        }
    }

    std::process::exit(exit_code);
}

fn read_file_reader<R: Read>(
    reader: R,
    sep: &[char],
    lines: &mut Vec<Vec<char>>,
) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut r = reader;
    r.read_to_string(&mut buf)?;
    let chars: Vec<char> = buf.chars().collect();
    let sep_len = sep.len();

    let mut start = 0usize;
    let mut i = 0usize;
    while i + sep_len <= chars.len() {
        if chars[i..i + sep_len] == sep[..] {
            lines.push(chars[start..i].to_vec());
            i += sep_len;
            start = i;
        } else {
            i += 1;
        }
    }
    if start <= chars.len() {
        lines.push(chars[start..].to_vec());
    }
    Ok(())
}
