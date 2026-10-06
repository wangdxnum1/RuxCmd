mod cli;

use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    let field1 = args.field1 - 1;
    let field2 = args.field2 - 1;
    let delimiter = args.delimiter.unwrap_or('\t');

    let mut exit_code = 0i32;

    let file1_lines = match read_file(&args.file1) {
        Ok(lines) => lines,
        Err(e) => {
            eprintln!("join: {}: {}", args.file1.display(), e);
            exit_code = 1;
            Vec::new()
        }
    };

    let file2_lines = match read_file(&args.file2) {
        Ok(lines) => lines,
        Err(e) => {
            eprintln!("join: {}: {}", args.file2.display(), e);
            exit_code = 1;
            Vec::new()
        }
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    let mut file2_map: HashMap<String, Vec<String>> = HashMap::new();
    for line in &file2_lines {
        let parts: Vec<&str> = line.split(delimiter).collect();
        if field2 < parts.len() {
            let key = parts[field2].to_string();
            file2_map.entry(key).or_default().push(line.clone());
        }
    }

    for line in &file1_lines {
        let parts: Vec<&str> = line.split(delimiter).collect();
        if field1 < parts.len() {
            let key = parts[field1].to_string();
            if let Some(matching_lines) = file2_map.get(&key) {
                for matching_line in matching_lines {
                    println!("{}{}{}", line, delimiter, matching_line);
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn read_file(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        lines.push(line.trim_end_matches('\n').to_string());
    }

    Ok(lines)
}
