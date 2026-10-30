mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("rev 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = BufReader::new(std::io::stdin());
        if let Err(e) = process_reader(stdin) {
            eprintln!("rev: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = BufReader::new(std::io::stdin());
                if let Err(e) = process_reader(stdin) {
                    eprintln!("rev: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_reader(reader) {
                            eprintln!("rev: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("rev: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_reader<R: BufRead>(mut reader: R) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut line = Vec::new();
    loop {
        line.clear();
        let bytes_read = reader.read_until(b'\n', &mut line)?;
        if bytes_read == 0 {
            break;
        }
        reverse_line(&line, &mut out)?;
    }
    Ok(())
}

/// Reverse one raw line: multibyte-aware over valid UTF-8 like util-linux
/// rev, byte-by-byte as a fallback for invalid sequences.
fn reverse_line(line: &[u8], out: &mut impl std::io::Write) -> Result<(), std::io::Error> {
    let content = if line.last() == Some(&b'\n') {
        &line[..line.len() - 1]
    } else {
        line
    };
    match std::str::from_utf8(content) {
        Ok(s) => {
            let reversed: String = s.chars().rev().collect();
            out.write_all(reversed.as_bytes())?;
        }
        Err(_) => {
            for &b in content.iter().rev() {
                out.write_all(&[b])?;
            }
        }
    }
    if line.last() == Some(&b'\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}
