mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("od 0.1.0");
        return;
    }

    let address_base = args.get_address_base();
    let display_type = args.get_display_type();
    let word_size = args.get_word_size();
    let words_per_line = args.get_words_per_line();

    let mut exit_code = 0i32;

    if let Some(path) = &args.file {
        let path_str = path.to_string_lossy();
        if path_str == "-" {
            let stdin = std::io::stdin();
            if let Err(e) = dump_reader(stdin, address_base, display_type, word_size, words_per_line) {
                eprintln!("od: error reading stdin: {}", e);
                exit_code = 1;
            }
        } else {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    if let Err(e) = dump_reader(reader, address_base, display_type, word_size, words_per_line) {
                        eprintln!("od: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("od: {}: {}", path.display(), e);
                    exit_code = 1;
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        if let Err(e) = dump_reader(stdin, address_base, display_type, word_size, words_per_line) {
            eprintln!("od: error reading stdin: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn format_address(addr: usize, base: &cli::AddressBase) -> String {
    match base {
        cli::AddressBase::Octal => format!("{:07o}", addr),
        cli::AddressBase::Decimal => format!("{:08}", addr),
        cli::AddressBase::Hexadecimal => format!("{:06x}", addr),
    }
}

fn format_byte(byte: u8, display_type: &cli::DisplayType) -> String {
    match display_type {
        cli::DisplayType::Octal => format!("{:03o}", byte),
        cli::DisplayType::Hexadecimal => format!("{:02x}", byte),
        cli::DisplayType::Decimal => format!("{:03}", byte),
        cli::DisplayType::Char => {
            if byte.is_ascii_graphic() || byte == b' ' {
                format!("{}", byte as char)
            } else if byte == b'\n' {
                "\\n".to_string()
            } else if byte == b'\r' {
                "\\r".to_string()
            } else if byte == b'\t' {
                "\\t".to_string()
            } else {
                format!("\\{:03o}", byte)
            }
        }
        cli::DisplayType::String => {
            if byte.is_ascii_graphic() || byte == b' ' {
                format!("{}", byte as char)
            } else {
                format!("\\{:03o}", byte)
            }
        }
        cli::DisplayType::Float => format!("{:02x}", byte),
    }
}

fn format_word(data: &[u8], word_size: &cli::WordSize, display_type: &cli::DisplayType) -> String {
    let size = match word_size {
        cli::WordSize::Byte => 1,
        cli::WordSize::TwoBytes => 2,
        cli::WordSize::FourBytes => 4,
        cli::WordSize::EightBytes => 8,
    };

    if data.len() < size {
        let mut result = String::new();
        for i in 0..size {
            if i < data.len() {
                result.push_str(&format_byte(data[i], display_type));
            } else {
                match display_type {
                    cli::DisplayType::Octal => result.push_str("   "),
                    cli::DisplayType::Hexadecimal => result.push_str("  "),
                    cli::DisplayType::Decimal => result.push_str("   "),
                    cli::DisplayType::Char | cli::DisplayType::String => result.push(' '),
                    cli::DisplayType::Float => result.push_str("  "),
                }
            }
            if i < size - 1 {
                result.push(' ');
            }
        }
        return result;
    }

    match (word_size, display_type) {
        (cli::WordSize::Byte, _) => format_byte(data[0], display_type),
        (cli::WordSize::TwoBytes, cli::DisplayType::Octal) => format!("{:06o}", u16::from_be_bytes([data[0], data[1]])),
        (cli::WordSize::TwoBytes, cli::DisplayType::Hexadecimal) => format!("{:04x}", u16::from_be_bytes([data[0], data[1]])),
        (cli::WordSize::TwoBytes, cli::DisplayType::Decimal) => format!("{:05}", u16::from_be_bytes([data[0], data[1]])),
        (cli::WordSize::FourBytes, cli::DisplayType::Octal) => format!("{:012o}", u32::from_be_bytes([data[0], data[1], data[2], data[3]])),
        (cli::WordSize::FourBytes, cli::DisplayType::Hexadecimal) => format!("{:08x}", u32::from_be_bytes([data[0], data[1], data[2], data[3]])),
        (cli::WordSize::FourBytes, cli::DisplayType::Decimal) => format!("{:010}", u32::from_be_bytes([data[0], data[1], data[2], data[3]])),
        (cli::WordSize::EightBytes, cli::DisplayType::Octal) => format!("{:024o}", u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])),
        (cli::WordSize::EightBytes, cli::DisplayType::Hexadecimal) => format!("{:016x}", u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])),
        (cli::WordSize::EightBytes, cli::DisplayType::Decimal) => format!("{:020}", u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])),
        (_, cli::DisplayType::Char) | (_, cli::DisplayType::String) => {
            let mut result = String::new();
            for &byte in data.iter().take(size) {
                result.push_str(&format_byte(byte, display_type));
            }
            result
        }
        (_, cli::DisplayType::Float) => {
            let mut result = String::new();
            for &byte in data.iter().take(size) {
                result.push_str(&format_byte(byte, display_type));
            }
            result
        }
    }
}

fn dump_reader<R: Read>(
    mut reader: R,
    address_base: cli::AddressBase,
    display_type: cli::DisplayType,
    word_size: cli::WordSize,
    words_per_line: usize,
) -> Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut offset = 0;
    let total = buffer.len();
    let size = match word_size {
        cli::WordSize::Byte => 1,
        cli::WordSize::TwoBytes => 2,
        cli::WordSize::FourBytes => 4,
        cli::WordSize::EightBytes => 8,
    };

    while offset < total {
        let line_size = std::cmp::min(words_per_line * size, total - offset);
        let line_data = &buffer[offset..offset + line_size];

        print!("{} ", format_address(offset, &address_base));

        let mut word_offset = 0;
        while word_offset < line_size {
            let word_end = std::cmp::min(word_offset + size, line_size);
            let word_data = &line_data[word_offset..word_end];
            let formatted = format_word(word_data, &word_size, &display_type);
            
            print!("{}", formatted);
            
            if word_offset + size < line_size {
                print!(" ");
            }
            
            word_offset += size;
        }

        if display_type != cli::DisplayType::Char {
            print!("  ");
            for &byte in line_data {
                if byte.is_ascii_graphic() {
                    print!("{}", byte as char);
                } else if byte == b'\n' {
                    print!("\\n");
                } else if byte == b'\r' {
                    print!("\\r");
                } else if byte == b'\t' {
                    print!("\\t");
                } else {
                    print!(".");
                }
            }
        }

        println!();
        offset += line_size;
    }

    Ok(())
}
