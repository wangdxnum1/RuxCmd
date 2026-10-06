mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("fold 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = BufReader::new(std::io::stdin());
        if let Err(e) = process_reader(stdin, &args) {
            eprintln!("fold: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = BufReader::new(std::io::stdin());
                if let Err(e) = process_reader(stdin, &args) {
                    eprintln!("fold: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_reader(reader, &args) {
                            eprintln!("fold: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("fold: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_reader<R: BufRead>(mut reader: R, args: &cli::Args) -> Result<(), std::io::Error> {
    let mut line = String::new();
    loop {
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let has_newline = line.ends_with('\n');
        let content = if has_newline {
            line.trim_end_matches('\n')
        } else {
            line.as_str()
        };

        fold_line(content, args);

        if has_newline {
            println!();
        }

        line.clear();
    }
    Ok(())
}

fn fold_line(line: &str, args: &cli::Args) {
    let width = args.width;
    if width == 0 {
        print!("{}", line);
        return;
    }

    if args.bytes {
        fold_by_bytes(line, width, args.spaces);
    } else {
        fold_by_chars(line, width, args.spaces);
    }
}

fn fold_by_chars(line: &str, width: usize, split_on_spaces: bool) {
    let chars: Vec<char> = line.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        let mut end = start + width;
        if end > chars.len() {
            end = chars.len();
        }

        if split_on_spaces && end < chars.len() {
            let mut split_pos = end;
            for i in (start..end).rev() {
                if chars[i].is_whitespace() {
                    split_pos = i + 1;
                    break;
                }
            }
            if split_pos != end {
                end = split_pos;
            }
        }

        let segment: String = chars[start..end].iter().collect();
        print!("{}", segment);

        start = end;

        if start < chars.len() {
            println!();
        }
    }
}

fn fold_by_bytes(line: &str, width: usize, split_on_spaces: bool) {
    let bytes = line.as_bytes();
    let mut start = 0;

    while start < bytes.len() {
        let mut end = start + width;
        if end > bytes.len() {
            end = bytes.len();
        }

        if split_on_spaces && end < bytes.len() {
            let mut split_pos = end;
            for i in (start..end).rev() {
                if bytes[i].is_ascii_whitespace() {
                    split_pos = i + 1;
                    break;
                }
            }
            if split_pos != end {
                end = split_pos;
            }
        }

        let segment = &bytes[start..end];
        print!("{}", String::from_utf8_lossy(segment));

        start = end;

        if start < bytes.len() {
            println!();
        }
    }
}
