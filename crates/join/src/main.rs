mod cli;

use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// How input lines are split into fields.
///
/// GNU join splits on runs of blanks (collapsing them, ignoring leading
/// whitespace) unless `-t` is given, in which case the single character is
/// used verbatim (consecutive occurrences yield empty fields).
#[derive(Clone, Copy)]
enum Separator {
    Whitespace,
    Char(char),
}

impl Separator {
    fn split<'a>(&self, line: &'a str) -> Vec<&'a str> {
        match self {
            Separator::Char(c) => line.split(*c).collect(),
            Separator::Whitespace => line.split_whitespace().collect(),
        }
    }

    /// The separator used between output fields.
    fn output_sep(&self) -> String {
        match self {
            Separator::Char(c) => c.to_string(),
            Separator::Whitespace => " ".to_string(),
        }
    }
}

fn main() {
    let args = cli::Args::parse();

    // Fields are numbered from 1 (GNU convention); 0 would underflow below.
    if args.field1 == 0 || args.field2 == 0 {
        let bad = if args.field1 == 0 {
            args.field1
        } else {
            args.field2
        };
        eprintln!("join: invalid field number '{}'", bad);
        std::process::exit(1);
    }

    let field1 = args.field1 - 1;
    let field2 = args.field2 - 1;
    let separator = match args.delimiter {
        Some(c) => Separator::Char(c),
        None => Separator::Whitespace,
    };
    let output_sep = separator.output_sep();

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

    // Key -> the fields of every matching file2 line (needed because a key
    // may occur on several file2 lines and each pair produces one output line).
    let mut file2_map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for line in &file2_lines {
        let parts = separator.split(line);
        if field2 < parts.len() {
            let key = parts[field2].to_string();
            let fields = parts.iter().map(|s| s.to_string()).collect();
            file2_map.entry(key).or_default().push(fields);
        }
    }

    // GNU/POSIX output shape: the join key, then file1's remaining fields,
    // then file2's remaining fields — file2's key field is NOT repeated.
    for line in &file1_lines {
        let parts = separator.split(line);
        if field1 < parts.len() {
            let key = parts[field1].to_string();
            if let Some(matching_lines) = file2_map.get(&key) {
                for fields2 in matching_lines {
                    let mut out_fields: Vec<String> =
                        Vec::with_capacity(parts.len() + fields2.len());
                    out_fields.push(key.clone());
                    for (i, field) in parts.iter().enumerate() {
                        if i != field1 {
                            out_fields.push((*field).to_string());
                        }
                    }
                    for (i, field) in fields2.iter().enumerate() {
                        if i != field2 {
                            out_fields.push(field.clone());
                        }
                    }
                    println!("{}", out_fields.join(&output_sep));
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
