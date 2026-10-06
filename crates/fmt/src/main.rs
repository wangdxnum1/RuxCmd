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

    let first_line_indent = args.crown_margin.unwrap_or(0);
    let subsequent_indent = args.crown_margin.map(|c| c / 2).unwrap_or(0);

    let mut current_line = String::new();
    let mut is_first_line = true;

    for word in words {
        let line_width = if is_first_line {
            args.width - first_line_indent
        } else {
            args.width - subsequent_indent
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
    let mut remaining = line;
    let available_width = width.saturating_sub(indent);

    while !remaining.is_empty() {
        if remaining.len() <= available_width {
            if indent > 0 {
                print!("{}", " ".repeat(indent));
            }
            println!("{}", remaining);
            break;
        }

        let mut split_pos = available_width;
        while split_pos > 0 && !remaining.is_char_boundary(split_pos) {
            split_pos -= 1;
        }

        if split_pos == 0 {
            split_pos = available_width;
            while !remaining.is_char_boundary(split_pos) {
                split_pos += 1;
            }
        }

        let (chunk, rest) = remaining.split_at(split_pos);
        let trimmed_chunk = chunk.trim_end();

        if indent > 0 {
            print!("{}", " ".repeat(indent));
        }
        println!("{}", trimmed_chunk);

        remaining = rest.trim_start();
    }
}
