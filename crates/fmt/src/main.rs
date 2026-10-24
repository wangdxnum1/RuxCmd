mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("fmt 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    for file in &args.files {
        let file_str = file.as_str();
        if file_str == "-" {
            let stdin = BufReader::new(std::io::stdin());
            if let Err(e) = process_reader(stdin, &args) {
                eprintln!("fmt: error reading stdin: {}", e);
                exit_code = 1;
            }
        } else {
            match File::open(file) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    if let Err(e) = process_reader(reader, &args) {
                        eprintln!("fmt: {}: {}", file_str, e);
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("fmt: {}: {}", file_str, e);
                    exit_code = 1;
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_reader<R: BufRead>(mut reader: R, args: &cli::Args) -> Result<(), std::io::Error> {
    let mut line = String::new();
    let mut paragraph: Vec<String> = Vec::new();

    loop {
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        let stripped = line.trim_end();
        let has_newline = line.ends_with('\n');

        if stripped.is_empty() {
            if !paragraph.is_empty() {
                format_paragraph(&paragraph, args);
                paragraph.clear();
            }
            if has_newline {
                println!();
            }
        } else {
            paragraph.push(stripped.to_string());
        }

        line.clear();
    }

    if !paragraph.is_empty() {
        format_paragraph(&paragraph, args);
    }

    Ok(())
}

/// Width in columns of a line's leading whitespace (tabs advance to the next
/// multiple of 8), used by crown-margin mode.
fn indent_width(line: &str) -> usize {
    let mut col = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => col += 1,
            '\t' => col = (col / 8 + 1) * 8,
            _ => break,
        }
    }
    col
}

fn format_paragraph(paragraph: &[String], args: &cli::Args) {
    if args.split_only {
        for line in paragraph {
            split_line(line, args.width, 0);
        }
        return;
    }

    let mut words: Vec<String> = Vec::new();
    for line in paragraph {
        words.extend(line.split_whitespace().map(|s| s.to_string()));
    }

    if words.is_empty() {
        return;
    }

    // Crown margin: the first output line keeps the first input line's
    // indent; every following line keeps the second input line's indent
    // (falling back to the first when the paragraph has a single line).
    let (first_line_indent, subsequent_indent) = if args.crown_margin {
        let first = paragraph.first().map(|l| indent_width(l)).unwrap_or(0);
        let second = paragraph.get(1).map(|l| indent_width(l)).unwrap_or(first);
        (first, second)
    } else {
        (0, 0)
    };

    // saturating_sub: the indent can exceed -w (underflow used to make
    // `width - indent` wrap around and silently disable wrapping).
    let first_line_width = args.width.saturating_sub(first_line_indent);
    let subsequent_line_width = args.width.saturating_sub(subsequent_indent);

    let mut current_line = String::new();
    let mut is_first_line = true;

    for word in words {
        let line_width = if is_first_line {
            first_line_width
        } else {
            subsequent_line_width
        };

        if current_line.is_empty() {
            current_line = word;
        } else if current_line.len() + 1 + word.len() <= line_width {
            current_line.push(' ');
            current_line.push_str(&word);
        } else {
            print_line(
                &current_line,
                is_first_line,
                first_line_indent,
                subsequent_indent,
            );
            current_line = word;
            is_first_line = false;
        }
    }

    if !current_line.is_empty() {
        print_line(
            &current_line,
            is_first_line,
            first_line_indent,
            subsequent_indent,
        );
    }
}

fn print_line(line: &str, is_first_line: bool, first_indent: usize, sub_indent: usize) {
    let indent = if is_first_line {
        first_indent
    } else {
        sub_indent
    };
    if indent > 0 {
        print!("{}", " ".repeat(indent));
    }
    println!("{}", line);
}

fn split_line(line: &str, width: usize, indent: usize) {
    let available_width = width.saturating_sub(indent);
    let mut remaining = line.to_string();

    while !remaining.is_empty() {
        if remaining.len() <= available_width {
            if indent > 0 {
                print!("{}", " ".repeat(indent));
            }
            println!("{}", remaining);
            break;
        }

        // Break at the last whitespace inside the width window so words are
        // not cut in half; only a window without any whitespace falls back
        // to a hard cut at a character boundary.
        let mut cut = available_width.min(remaining.len());
        while cut > 0 && !remaining.is_char_boundary(cut) {
            cut -= 1;
        }
        let head = &remaining[..cut];

        let (chunk, next): (String, String) = match head.rfind(char::is_whitespace) {
            Some(pos) => {
                let chunk = head[..pos].trim_end().to_string();
                // The tail after the last whitespace continues the same word
                // beyond the window, so it carries over intact.
                let mut next = head[pos..].trim_start().to_string();
                next.push_str(&remaining[cut..]);
                (chunk, next)
            }
            None => (head.trim_end().to_string(), remaining[cut..].to_string()),
        };

        if indent > 0 {
            print!("{}", " ".repeat(indent));
        }
        println!("{}", chunk);
        remaining = next;
    }
}
