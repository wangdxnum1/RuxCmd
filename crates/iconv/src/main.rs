mod cli;

use clap::Parser;
use encoding_rs::Encoding;
use std::fs;
use std::io::{Read, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("iconv 0.1.0");
        return;
    }

    if args.list {
        list_encodings();
        return;
    }

    let from_code = args.from_code.as_deref().unwrap_or("");
    let to_code = args.to_code.as_deref().unwrap_or("");

    if from_code.is_empty() || to_code.is_empty() {
        eprintln!("iconv: missing from-code or to-code");
        std::process::exit(1);
    }

    let from_encoding = match Encoding::for_label(from_code.as_bytes()) {
        Some(e) => e,
        None => {
            eprintln!("iconv: invalid from-code '{}'", from_code);
            std::process::exit(1);
        }
    };

    let to_encoding = match Encoding::for_label(to_code.as_bytes()) {
        Some(e) => e,
        None => {
            eprintln!("iconv: invalid to-code '{}'", to_code);
            std::process::exit(1);
        }
    };

    let mut exit_code = 0i32;

    let mut all_input = Vec::new();

    if args.files.is_empty() {
        if let Err(e) = std::io::stdin().read_to_end(&mut all_input) {
            eprintln!("iconv: error reading stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                if let Err(e) = std::io::stdin().read_to_end(&mut all_input) {
                    eprintln!("iconv: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match fs::read(path) {
                    Ok(data) => all_input.extend(data),
                    Err(e) => {
                        eprintln!("iconv: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    let (result, _, _) = from_encoding.decode(&all_input);
    let (output_bytes, _, _) = to_encoding.encode(&result);

    if let Some(out_path) = &args.output {
        if let Err(e) = fs::write(out_path, &output_bytes) {
            eprintln!("iconv: {}: {}", out_path.display(), e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = std::io::stdout().write_all(&output_bytes) {
            eprintln!("iconv: write error: {}", e);
            std::process::exit(1);
        }
    }

    std::process::exit(exit_code);
}

fn list_encodings() {
    let encodings = [
        ("UTF-8", "Unicode (UTF-8)"),
        ("UTF-16", "Unicode (UTF-16 with BOM)"),
        ("UTF-16BE", "Unicode (UTF-16 big-endian)"),
        ("UTF-16LE", "Unicode (UTF-16 little-endian)"),
        ("GBK", "GBK (Chinese)"),
        ("GB2312", "GB2312 (Chinese simplified)"),
        ("GB18030", "GB18030 (Chinese)"),
        ("ISO-8859-1", "ISO-8859-1 (Latin-1)"),
        ("US-ASCII", "US-ASCII"),
        ("windows-1252", "Windows-1252"),
        ("Shift_JIS", "Shift_JIS (Japanese)"),
        ("EUC-JP", "EUC-JP (Japanese)"),
        ("ISO-2022-JP", "ISO-2022-JP (Japanese)"),
        ("Big5", "Big5 (Traditional Chinese)"),
        ("EUC-KR", "EUC-KR (Korean)"),
        ("ISO-8859-2", "ISO-8859-2 (Latin-2)"),
        ("ISO-8859-3", "ISO-8859-3 (Latin-3)"),
        ("ISO-8859-4", "ISO-8859-4 (Latin-4)"),
        ("ISO-8859-5", "ISO-8859-5 (Cyrillic)"),
        ("ISO-8859-6", "ISO-8859-6 (Arabic)"),
        ("ISO-8859-7", "ISO-8859-7 (Greek)"),
        ("ISO-8859-8", "ISO-8859-8 (Hebrew)"),
        ("ISO-8859-9", "ISO-8859-9 (Latin-5)"),
        ("ISO-8859-10", "ISO-8859-10 (Latin-6)"),
        ("ISO-8859-13", "ISO-8859-13 (Latin-7)"),
        ("ISO-8859-14", "ISO-8859-14 (Latin-8)"),
        ("ISO-8859-15", "ISO-8859-15 (Latin-9)"),
        ("ISO-8859-16", "ISO-8859-16 (Latin-10)"),
    ];

    for (name, desc) in encodings {
        println!("{} - {}", name, desc);
    }
}
