mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("strings 0.1.0");
        return;
    }

    let min_length = args.min_length;
    let encoding = args.encoding;
    let files = args.files;

    let mut exit_code = 0i32;

    if files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = strings_reader(stdin, min_length, encoding) {
            eprintln!("strings: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = strings_reader(stdin, min_length, encoding) {
                    eprintln!("strings: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = strings_reader(reader, min_length, encoding) {
                            eprintln!("strings: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("strings: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn is_printable_seven_bit(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0x7E
}

fn is_printable_eight_bit(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0xFE
}

fn strings_reader<R: Read>(mut reader: R, min_length: usize, encoding: Option<cli::Encoding>) -> Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let encoding = encoding.unwrap_or(cli::Encoding::SevenBit);

    match encoding {
        cli::Encoding::SevenBit => extract_seven_bit(&buffer, min_length),
        cli::Encoding::EightBit => extract_eight_bit(&buffer, min_length),
        cli::Encoding::Utf16BE => extract_utf16_be(&buffer, min_length),
        cli::Encoding::Utf16LE => extract_utf16_le(&buffer, min_length),
        cli::Encoding::Utf32BE => extract_utf32_be(&buffer, min_length),
        cli::Encoding::Utf32LE => extract_utf32_le(&buffer, min_length),
    }

    Ok(())
}

fn extract_seven_bit(buffer: &[u8], min_length: usize) {
    let mut current_string = Vec::new();

    for &byte in buffer {
        if is_printable_seven_bit(byte) || byte == b'\n' || byte == b'\r' {
            if byte == b'\n' || byte == b'\r' {
                if current_string.len() >= min_length {
                    println!("{}", String::from_utf8_lossy(&current_string));
                }
                current_string.clear();
            } else {
                current_string.push(byte);
            }
        } else {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf8_lossy(&current_string));
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", String::from_utf8_lossy(&current_string));
    }
}

fn extract_eight_bit(buffer: &[u8], min_length: usize) {
    let mut current_string = Vec::new();

    for &byte in buffer {
        if is_printable_eight_bit(byte) || byte == b'\n' || byte == b'\r' {
            if byte == b'\n' || byte == b'\r' {
                if current_string.len() >= min_length {
                    println!("{}", String::from_utf8_lossy(&current_string));
                }
                current_string.clear();
            } else {
                current_string.push(byte);
            }
        } else {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf8_lossy(&current_string));
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", String::from_utf8_lossy(&current_string));
    }
}

fn extract_utf16_be(buffer: &[u8], min_length: usize) {
    let mut current_string = Vec::new();

    let mut i = 0;
    while i + 1 < buffer.len() {
        let ch = ((buffer[i] as u32) << 8) | (buffer[i + 1] as u32);
        i += 2;

        if ch == 0x000A || ch == 0x000D {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf16_lossy(&current_string));
            }
            current_string.clear();
        } else if is_printable_unicode(ch) {
            current_string.push(ch as u16);
        } else {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf16_lossy(&current_string));
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", String::from_utf16_lossy(&current_string));
    }
}

fn extract_utf16_le(buffer: &[u8], min_length: usize) {
    let mut current_string = Vec::new();

    let mut i = 0;
    while i + 1 < buffer.len() {
        let ch = ((buffer[i + 1] as u32) << 8) | (buffer[i] as u32);
        i += 2;

        if ch == 0x000A || ch == 0x000D {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf16_lossy(&current_string));
            }
            current_string.clear();
        } else if is_printable_unicode(ch) {
            current_string.push(ch as u16);
        } else {
            if current_string.len() >= min_length {
                println!("{}", String::from_utf16_lossy(&current_string));
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", String::from_utf16_lossy(&current_string));
    }
}

fn extract_utf32_be(buffer: &[u8], min_length: usize) {
    let mut current_string = String::new();

    let mut i = 0;
    while i + 3 < buffer.len() {
        let ch = ((buffer[i] as u32) << 24)
            | ((buffer[i + 1] as u32) << 16)
            | ((buffer[i + 2] as u32) << 8)
            | (buffer[i + 3] as u32);
        i += 4;

        if ch == 0x0000000A || ch == 0x0000000D {
            if current_string.len() >= min_length {
                println!("{}", current_string);
            }
            current_string.clear();
        } else if is_printable_unicode(ch) {
            if let Some(c) = std::char::from_u32(ch) {
                current_string.push(c);
            } else {
                if current_string.len() >= min_length {
                    println!("{}", current_string);
                }
                current_string.clear();
            }
        } else {
            if current_string.len() >= min_length {
                println!("{}", current_string);
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", current_string);
    }
}

fn extract_utf32_le(buffer: &[u8], min_length: usize) {
    let mut current_string = String::new();

    let mut i = 0;
    while i + 3 < buffer.len() {
        let ch = ((buffer[i + 3] as u32) << 24)
            | ((buffer[i + 2] as u32) << 16)
            | ((buffer[i + 1] as u32) << 8)
            | (buffer[i] as u32);
        i += 4;

        if ch == 0x0000000A || ch == 0x0000000D {
            if current_string.len() >= min_length {
                println!("{}", current_string);
            }
            current_string.clear();
        } else if is_printable_unicode(ch) {
            if let Some(c) = std::char::from_u32(ch) {
                current_string.push(c);
            } else {
                if current_string.len() >= min_length {
                    println!("{}", current_string);
                }
                current_string.clear();
            }
        } else {
            if current_string.len() >= min_length {
                println!("{}", current_string);
            }
            current_string.clear();
        }
    }

    if current_string.len() >= min_length {
        println!("{}", current_string);
    }
}

fn is_printable_unicode(ch: u32) -> bool {
    if ch == 0x0020 {
        return true;
    }

    let c = match std::char::from_u32(ch) {
        Some(c) => c,
        None => return false,
    };

    c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_whitespace() || (ch >= 0x20 && ch <= 0x7E)
}