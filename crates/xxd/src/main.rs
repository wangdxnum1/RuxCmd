use clap::Parser;
use std::io::{Read, Write};

/// Make a hexdump or do the reverse — a Windows port of Linux xxd.
#[derive(Parser, Debug)]
#[command(name = "xxd", version, about)]
pub struct Args {
    /// Reverse operation: convert hex dump back to binary
    #[arg(short = 'r', long = "revert")]
    pub revert: bool,

    /// Bytes per line (default 16)
    #[arg(short = 'c', long = "cols", value_name = "COLS", default_value_t = 16)]
    pub cols: usize,

    /// Read from FILE instead of stdin
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let mut input = Vec::new();

    for file in &args.files {
        if file == "-" {
            let _ = std::io::stdin().read_to_end(&mut input);
        } else {
            match std::fs::read(file) {
                Ok(mut data) => input.append(&mut data),
                Err(e) => {
                    eprintln!("xxd: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }
    if args.files.is_empty() {
        let _ = std::io::stdin().read_to_end(&mut input);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if args.revert {
        // Strip whitespace and addresses, then decode hex pairs.
        let text = String::from_utf8_lossy(&input);
        let mut hex = String::new();
        for line in text.lines() {
            let line = line.trim();
            // Skip the address prefix (8 hex digits followed by ':')
            if let Some(pos) = line.find(':') {
                let rest = &line[pos + 1..];
                for c in rest.chars() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                    } else if c != ' ' {
                        break;
                    }
                }
            } else {
                for c in line.chars() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                    }
                }
            }
        }
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap_or(0))
            .collect();
        let _ = out.write_all(&bytes);
    } else {
        let cols = args.cols.max(1);
        for (chunk_idx, chunk) in input.chunks(cols).enumerate() {
            // Address in hex
            write!(out, "{:08x}:", chunk_idx * cols).unwrap();

            // Hex column
            for i in 0..cols {
                if i % 2 == 0 {
                    write!(out, " ").unwrap();
                }
                if i < chunk.len() {
                    write!(out, "{:02x}", chunk[i]).unwrap();
                } else {
                    write!(out, "  ").unwrap();
                }
            }

            // ASCII column
            write!(out, "  ").unwrap();
            for &b in chunk {
                if (0x20..0x7f).contains(&b) {
                    write!(out, "{}", b as char).unwrap();
                } else {
                    write!(out, ".").unwrap();
                }
            }
            writeln!(out).unwrap();
        }
    }
    let _ = out.flush();
}
