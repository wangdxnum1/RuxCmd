use clap::Parser;
use std::io::{Read, Write};

/// Encode or decode base32 (RFC 4648) - a Windows port of Linux base32.
#[derive(Parser, Debug)]
#[command(name = "base32", version, about)]
pub struct Args {
    /// Decode data
    #[arg(short = 'd', long = "decode")]
    pub decode: bool,

    /// Ignore non-alphabet characters when decoding
    #[arg(short = 'i', long = "ignore-garbage")]
    pub ignore_garbage: bool,

    /// Wrap encoded lines after COLS characters (0 disables wrapping)
    #[arg(short = 'w', long = "wrap", value_name = "COLS", default_value_t = 76)]
    pub wrap: usize,

    /// Files to process (default: standard input)
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn main() {
    let args = Args::parse();
    let mut input: Vec<u8> = Vec::new();

    if args.files.is_empty() {
        if let Err(e) = std::io::stdin().read_to_end(&mut input) {
            eprintln!("base32: error reading stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        for file in &args.files {
            let path = if file == "-" {
                if let Err(e) = std::io::stdin().read_to_end(&mut input) {
                    eprintln!("base32: error reading stdin: {}", e);
                    std::process::exit(1);
                }
                continue;
            } else {
                file.clone()
            };
            match std::fs::read(&path) {
                Ok(mut data) => input.append(&mut data),
                Err(e) => {
                    eprintln!("base32: {}: {}", path, e);
                    std::process::exit(1);
                }
            }
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let result = if args.decode {
        decode(&input, args.ignore_garbage, &mut out)
    } else {
        encode(&input, args.wrap, &mut out)
    };
    if let Err(message) = result {
        eprintln!("base32: {}", message);
        std::process::exit(1);
    }
    let _ = out.flush();
}

fn encode(data: &[u8], wrap: usize, out: &mut impl Write) -> Result<(), String> {
    let mut column = 0usize;
    for chunk in data.chunks(5) {
        let mut group = [0u8; 5];
        group[..chunk.len()].copy_from_slice(chunk);
        let bits = u64::from(group[0]) << 32
            | u64::from(group[1]) << 24
            | u64::from(group[2]) << 16
            | u64::from(group[3]) << 8
            | u64::from(group[4]);
        let symbols: [u8; 8] = [
            ALPHABET[((bits >> 35) & 31) as usize],
            ALPHABET[((bits >> 30) & 31) as usize],
            ALPHABET[((bits >> 25) & 31) as usize],
            ALPHABET[((bits >> 20) & 31) as usize],
            ALPHABET[((bits >> 15) & 31) as usize],
            ALPHABET[((bits >> 10) & 31) as usize],
            ALPHABET[((bits >> 5) & 31) as usize],
            ALPHABET[(bits & 31) as usize],
        ];
        let keep = match chunk.len() {
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => 8,
        };
        for &symbol in &symbols[..keep] {
            out.write_all(&[symbol]).map_err(|e| e.to_string())?;
            column += 1;
            if wrap > 0 && column == wrap {
                out.write_all(b"\n").map_err(|e| e.to_string())?;
                column = 0;
            }
        }
        for _ in keep..8 {
            out.write_all(b"=").map_err(|e| e.to_string())?;
            column += 1;
            if wrap > 0 && column == wrap {
                out.write_all(b"\n").map_err(|e| e.to_string())?;
                column = 0;
            }
        }
    }
    // GNU -w 0 emits no trailing newline; wrapped output that just wrapped
    // already has one.
    if wrap > 0 && column != 0 {
        out.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn decode(data: &[u8], ignore_garbage: bool, out: &mut impl Write) -> Result<(), String> {
    let mut buffer: Vec<u8> = Vec::new();
    for &byte in data {
        if byte.is_ascii_alphanumeric() || byte == b'=' {
            // Digits 0, 1, 8, 9 are not in the base32 alphabet.
            if byte.is_ascii_digit() && !matches!(byte, b'2'..=b'7') {
                if !ignore_garbage {
                    return Err("invalid input".to_string());
                }
                continue;
            }
            buffer.push(byte);
        } else if byte == b'\n' || byte == b'\r' {
            // Newlines are the encoder's line wrapping; always tolerated.
            continue;
        } else if !ignore_garbage {
            return Err("invalid input".to_string());
        }
    }

    // Padding is dropped; symbols are processed in groups of 8 across the
    // whole stream, so multi-group inputs decode correctly.
    let mut group: Vec<u8> = Vec::new();
    for &symbol in &buffer {
        if symbol == b'=' {
            continue;
        }
        group.push(symbol);
        if group.len() == 8 {
            emit_group(&group, out)?;
            group.clear();
        }
    }
    if !group.is_empty() {
        if group.len() < 2 {
            return Err("invalid input".to_string());
        }
        let mut padded = group.clone();
        while padded.len() < 8 {
            padded.push(ALPHABET[0]);
        }
        let keep = match group.len() {
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            _ => return Err("invalid input".to_string()),
        };
        let mut bytes = vec![0u8; keep];
        capture_group(&padded, &mut bytes);
        out.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn emit_group(group: &[u8], out: &mut impl Write) -> Result<(), String> {
    let mut bytes = [0u8; 5];
    capture_group(group, &mut bytes);
    out.write_all(&bytes).map_err(|e| e.to_string())
}

fn capture_group(group: &[u8], bytes: &mut [u8]) {
    let mut bits: u64 = 0;
    for &symbol in group {
        let value = if symbol == b'=' {
            0
        } else {
            symbol_value(symbol)
        };
        bits = (bits << 5) | value as u64;
    }
    bits <<= 40 - 5 * group.len();
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = ((bits >> (32 - 8 * i)) & 0xFF) as u8;
    }
}

fn symbol_value(symbol: u8) -> u8 {
    match symbol {
        b'A'..=b'Z' => symbol - b'A',
        b'2'..=b'7' => symbol - b'2' + 26,
        b'a'..=b'z' => symbol - b'a',
        _ => 0,
    }
}
