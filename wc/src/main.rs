mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

struct Counts {
    lines: u64,
    words: u64,
    bytes: u64,
    chars: u64,
    max_line_length: usize,
}

fn main() {
    let args = cli::Args::parse();

    let show_lines = args.lines || (!args.words && !args.bytes && !args.chars && !args.max_line_length);
    let show_words = args.words || (!args.lines && !args.bytes && !args.chars && !args.max_line_length);
    let show_bytes = args.bytes || (!args.lines && !args.words && !args.chars && !args.max_line_length);
    let show_chars = args.chars;
    let show_max_line = args.max_line_length;

    let mut total_counts = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
        max_line_length: 0,
    };

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        let counts = wc_reader(stdin.lock(), show_chars);
        print_counts(&counts, "<stdin>", show_lines, show_words, show_bytes, show_chars, show_max_line);
    } else {
        let multiple_files = args.files.len() > 1;

        for file in &args.files {
            let file_name = file.display().to_string();
            let counts = match wc_file(file, show_chars) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("wc: {}: {}", file_name, e);
                    continue;
                }
            };

            print_counts(&counts, &file_name, show_lines, show_words, show_bytes, show_chars, show_max_line);

            total_counts.lines += counts.lines;
            total_counts.words += counts.words;
            total_counts.bytes += counts.bytes;
            total_counts.chars += counts.chars;
            total_counts.max_line_length = total_counts.max_line_length.max(counts.max_line_length);
        }

        if multiple_files {
            print_counts(&total_counts, "total", show_lines, show_words, show_bytes, show_chars, show_max_line);
        }
    }
}

fn wc_file(path: &Path, count_chars: bool) -> Result<Counts, String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    Ok(wc_reader(file, count_chars))
}

fn wc_reader<R: Read>(reader: R, count_chars: bool) -> Counts {
    let mut reader = BufReader::new(reader);
    let mut counts = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
        max_line_length: 0,
    };

    let mut in_word = false;
    let mut current_line_length = 0;

    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                counts.bytes += n as u64;

                for &byte in &buf[..n] {
                    counts.chars += 1;
                    current_line_length += 1;

                    if byte == b'\n' {
                        counts.lines += 1;
                        counts.max_line_length = counts.max_line_length.max(current_line_length);
                        current_line_length = 0;
                        in_word = false;
                    } else if byte.is_ascii_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        in_word = true;
                        counts.words += 1;
                    }
                }
            }
            Err(_) => break,
        }
    }

    if current_line_length > 0 {
        counts.max_line_length = counts.max_line_length.max(current_line_length);
    }

    counts
}

fn print_counts(counts: &Counts, name: &str, show_lines: bool, show_words: bool, show_bytes: bool, show_chars: bool, show_max_line: bool) {
    let mut output = Vec::new();

    if show_lines {
        output.push(format!("{}", counts.lines));
    }
    if show_words {
        output.push(format!("{}", counts.words));
    }
    if show_bytes {
        output.push(format!("{}", counts.bytes));
    }
    if show_chars {
        output.push(format!("{}", counts.chars));
    }
    if show_max_line {
        output.push(format!("{}", counts.max_line_length));
    }

    if !output.is_empty() {
        println!("{} {}", output.join(" "), name);
    }
}