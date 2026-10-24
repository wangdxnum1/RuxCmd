mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufReader, Read};
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

    let show_lines =
        args.lines || (!args.words && !args.bytes && !args.chars && !args.max_line_length);
    let show_words =
        args.words || (!args.lines && !args.bytes && !args.chars && !args.max_line_length);
    let show_bytes =
        args.bytes || (!args.lines && !args.words && !args.chars && !args.max_line_length);
    let show_chars = args.chars;
    let show_max_line = args.max_line_length;

    let mut total_counts = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
        max_line_length: 0,
    };
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        match wc_reader(stdin.lock(), show_chars) {
            Ok(counts) => print_counts(
                &counts,
                "<stdin>",
                show_lines,
                show_words,
                show_bytes,
                show_chars,
                show_max_line,
            ),
            Err(e) => {
                eprintln!("wc: <stdin>: {}", e);
                exit_code = 1;
            }
        }
    } else {
        let multiple_files = args.files.len() > 1;

        for file in &args.files {
            let file_name = file.display().to_string();
            let counts = match wc_file(file, show_chars) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("wc: {}: {}", file_name, e);
                    exit_code = 1;
                    continue;
                }
            };

            print_counts(
                &counts,
                &file_name,
                show_lines,
                show_words,
                show_bytes,
                show_chars,
                show_max_line,
            );

            total_counts.lines += counts.lines;
            total_counts.words += counts.words;
            total_counts.bytes += counts.bytes;
            total_counts.chars += counts.chars;
            total_counts.max_line_length = total_counts.max_line_length.max(counts.max_line_length);
        }

        if multiple_files {
            print_counts(
                &total_counts,
                "total",
                show_lines,
                show_words,
                show_bytes,
                show_chars,
                show_max_line,
            );
        }
    }

    use std::io::Write;
    let _ = std::io::stdout().flush();
    std::process::exit(exit_code);
}

fn wc_file(path: &Path, count_chars: bool) -> std::io::Result<Counts> {
    let file = fs::File::open(path)?;
    wc_reader(file, count_chars)
}

/// Incremental UTF-8 character counter for `-m`.
///
/// Sequences split across read chunks are carried over via `pending`. Any
/// invalid byte marks the whole stream invalid; the caller then falls back to
/// the byte count (GNU wc reports an error instead — declared deviation in
/// docs/compatibility.md).
struct CharCounter {
    pending: Vec<u8>,
    count: u64,
    invalid: bool,
}

impl CharCounter {
    fn new() -> Self {
        CharCounter {
            pending: Vec::new(),
            count: 0,
            invalid: false,
        }
    }

    fn feed(&mut self, chunk: &[u8]) {
        if self.invalid {
            return;
        }
        let mut merged: Vec<u8>;
        let data: &[u8] = if self.pending.is_empty() {
            chunk
        } else {
            merged = std::mem::take(&mut self.pending);
            merged.extend_from_slice(chunk);
            &merged
        };

        let mut i = 0usize;
        while i < data.len() {
            let Some(len) = utf8_sequence_len(data[i]) else {
                self.invalid = true;
                return;
            };
            if i + len > data.len() {
                // Incomplete trailing sequence: wait for the next chunk.
                self.pending.extend_from_slice(&data[i..]);
                return;
            }
            if is_valid_utf8_sequence(&data[i..i + len]) {
                self.count += 1;
            } else {
                self.invalid = true;
                return;
            }
            i += len;
        }
    }

    fn finish(&mut self) {
        if !self.pending.is_empty() {
            // Truncated sequence at end of input.
            self.invalid = true;
            self.pending.clear();
        }
    }
}

fn utf8_sequence_len(lead: u8) -> Option<usize> {
    match lead {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

/// Validate a full sequence whose length was derived by `utf8_sequence_len`
/// (same rules as Rust's UTF-8 decoder: rejects stray/overlong leads,
/// surrogates, and out-of-range code points).
fn is_valid_utf8_sequence(seq: &[u8]) -> bool {
    let cont = |b: u8| (b & 0xC0) == 0x80;
    match seq[0] {
        0x00..=0x7F => true,
        0xC2..=0xDF => cont(seq[1]),
        0xE0 => (0xA0..=0xBF).contains(&seq[1]),
        0xED => (0x80..=0x9F).contains(&seq[1]),
        0xE1..=0xEC | 0xEE..=0xEF => cont(seq[1]) && cont(seq[2]),
        0xF0 => (0x90..=0xBF).contains(&seq[1]) && cont(seq[2]) && cont(seq[3]),
        0xF4 => (0x80..=0x8F).contains(&seq[1]) && cont(seq[2]) && cont(seq[3]),
        0xF1..=0xF3 => cont(seq[1]) && cont(seq[2]) && cont(seq[3]),
        _ => false,
    }
}

fn wc_reader<R: Read>(reader: R, count_chars: bool) -> std::io::Result<Counts> {
    let mut reader = BufReader::new(reader);
    let mut counts = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
        max_line_length: 0,
    };
    let mut char_counter = CharCounter::new();

    let mut in_word = false;
    let mut current_line_length = 0;

    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                counts.bytes += n as u64;
                if count_chars {
                    char_counter.feed(&buf[..n]);
                }

                for &byte in &buf[..n] {
                    if byte == b'\n' {
                        counts.lines += 1;
                        counts.max_line_length = counts.max_line_length.max(current_line_length);
                        current_line_length = 0;
                        in_word = false;
                    } else {
                        // Line length excludes the newline byte (GNU -L).
                        current_line_length += 1;
                        if byte.is_ascii_whitespace() {
                            in_word = false;
                        } else if !in_word {
                            in_word = true;
                            counts.words += 1;
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }

    if current_line_length > 0 {
        counts.max_line_length = counts.max_line_length.max(current_line_length);
    }

    if count_chars {
        char_counter.finish();
        counts.chars = if char_counter.invalid {
            counts.bytes
        } else {
            char_counter.count
        };
    }

    Ok(counts)
}

fn print_counts(
    counts: &Counts,
    name: &str,
    show_lines: bool,
    show_words: bool,
    show_bytes: bool,
    show_chars: bool,
    show_max_line: bool,
) {
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
