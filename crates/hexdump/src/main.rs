mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, stdin};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("hexdump 0.1.0");
        return;
    }

    let mut buffer = Vec::new();

    if let Some(path) = &args.file {
        let path_str = path.to_string_lossy();
        if path_str == "-" {
            if let Err(e) = stdin().read_to_end(&mut buffer) {
                eprintln!("hexdump: error reading stdin: {}", e);
                std::process::exit(1);
            }
        } else {
            match File::open(path) {
                Ok(mut file) => {
                    if let Err(e) = file.read_to_end(&mut buffer) {
                        eprintln!("hexdump: {}: {}", path.display(), e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("hexdump: {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        if let Err(e) = stdin().read_to_end(&mut buffer) {
            eprintln!("hexdump: error reading stdin: {}", e);
            std::process::exit(1);
        }
    }

    let start = args.skip.unwrap_or(0);
    let len = args.length.unwrap_or(buffer.len());
    let end = std::cmp::min(start + len, buffer.len());

    if start >= buffer.len() {
        return;
    }

    let data = &buffer[start..end];

    if args.canonical {
        dump_canonical(data, start);
    } else if args.decimal {
        dump_decimal(data, start);
    } else if args.octal {
        dump_octal(data, start);
    } else if args.ascii {
        dump_ascii(data, start);
    } else {
        dump_hexadecimal(data, start);
    }
}

fn dump_canonical(data: &[u8], start: usize) {
    let mut offset = 0;
    while offset < data.len() {
        let line_size = std::cmp::min(16, data.len() - offset);
        let line_data = &data[offset..offset + line_size];

        print!("{:08x}  ", start + offset);

        for i in 0..16 {
            if i < line_size {
                print!("{:02x} ", line_data[i]);
            } else {
                print!("   ");
            }
            if i == 7 {
                print!(" ");
            }
        }

        print!(" |");
        for &byte in line_data {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");

        offset += line_size;
    }

    if !data.is_empty() {
        println!("{:08x}", start + data.len());
    }
}

fn dump_hexadecimal(data: &[u8], start: usize) {
    let mut offset = 0;
    while offset < data.len() {
        let line_size = std::cmp::min(16, data.len() - offset);
        let line_data = &data[offset..offset + line_size];

        print!("{:08x}  ", start + offset);

        let mut i = 0;
        while i < line_size {
            let size = std::cmp::min(2, line_size - i);
            if size == 2 {
                print!(
                    "{:04x} ",
                    u16::from_be_bytes([line_data[i], line_data[i + 1]])
                );
            } else {
                print!("{:02x}   ", line_data[i]);
            }
            i += 2;
            if i % 8 == 0 {
                print!(" ");
            }
        }

        println!();
        offset += line_size;
    }
}

fn dump_decimal(data: &[u8], start: usize) {
    let mut offset = 0;
    while offset < data.len() {
        let line_size = std::cmp::min(16, data.len() - offset);
        let line_data = &data[offset..offset + line_size];

        print!("{:08x}  ", start + offset);

        let mut i = 0;
        while i < line_size {
            let size = std::cmp::min(2, line_size - i);
            if size == 2 {
                print!(
                    "{:05} ",
                    u16::from_be_bytes([line_data[i], line_data[i + 1]])
                );
            } else {
                print!("{:03}   ", line_data[i]);
            }
            i += 2;
            if i % 8 == 0 {
                print!(" ");
            }
        }

        println!();
        offset += line_size;
    }
}

fn dump_octal(data: &[u8], start: usize) {
    let mut offset = 0;
    while offset < data.len() {
        let line_size = std::cmp::min(16, data.len() - offset);
        let line_data = &data[offset..offset + line_size];

        print!("{:08x}  ", start + offset);

        let mut i = 0;
        while i < line_size {
            let size = std::cmp::min(2, line_size - i);
            if size == 2 {
                print!(
                    "{:06o} ",
                    u16::from_be_bytes([line_data[i], line_data[i + 1]])
                );
            } else {
                print!("{:03o}   ", line_data[i]);
            }
            i += 2;
            if i % 8 == 0 {
                print!(" ");
            }
        }

        println!();
        offset += line_size;
    }
}

fn dump_ascii(data: &[u8], start: usize) {
    let mut offset = 0;
    while offset < data.len() {
        let line_size = std::cmp::min(16, data.len() - offset);
        let line_data = &data[offset..offset + line_size];

        print!("{:08x}  ", start + offset);

        for &byte in line_data {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{} ", byte as char);
            } else if byte == b'\n' {
                print!("\\n ");
            } else if byte == b'\r' {
                print!("\\r ");
            } else if byte == b'\t' {
                print!("\\t ");
            } else {
                print!("{:03o} ", byte);
            }
        }

        println!();
        offset += line_size;
    }
}
