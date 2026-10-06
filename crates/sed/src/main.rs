mod cli;

use clap::Parser;
use cli::Args;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn parse_substitute_command(expr: &str) -> Option<(String, String, bool)> {
    if !expr.starts_with('s') {
        return None;
    }

    let chars: Vec<char> = expr.chars().collect();
    let mut i = 1;
    if i >= chars.len() {
        return None;
    }

    let delimiter = chars[i];
    i += 1;

    let mut pattern = String::new();
    while i < chars.len() && chars[i] != delimiter {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 1;
            pattern.push(chars[i]);
        } else {
            pattern.push(chars[i]);
        }
        i += 1;
    }

    if i >= chars.len() || chars[i] != delimiter {
        return None;
    }
    i += 1;

    let mut replacement = String::new();
    while i < chars.len() && chars[i] != delimiter {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 1;
            replacement.push(chars[i]);
        } else {
            replacement.push(chars[i]);
        }
        i += 1;
    }

    if i >= chars.len() || chars[i] != delimiter {
        return None;
    }
    i += 1;

    let global = i < chars.len() && chars[i] == 'g';

    Some((pattern, replacement, global))
}

fn apply_expression(line: &str, expr: &str) -> String {
    if let Some((pattern, replacement, global)) = parse_substitute_command(expr) {
        if global {
            line.replace(&pattern, &replacement)
        } else {
            line.replacen(&pattern, &replacement, 1)
        }
    } else {
        line.to_string()
    }
}

fn process_stdin(args: &Args) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let mut line = line?;
        for expr in &args.expressions {
            line = apply_expression(&line, expr);
        }
        println!("{}", line);
    }

    Ok(())
}

fn process_file(args: &Args, file: &Path) -> io::Result<()> {
    let content = fs::read_to_string(file)?;
    let mut result = String::new();

    for line in content.lines() {
        let mut line = line.to_string();
        for expr in &args.expressions {
            line = apply_expression(&line, expr);
        }
        result.push_str(&line);
        result.push('\n');
    }

    if let Some(suffix) = &args.in_place {
        if !suffix.is_empty() {
            let backup_path = file.with_file_name(format!(
                "{}{}",
                file.file_name().unwrap().to_string_lossy(),
                suffix
            ));
            fs::copy(file, &backup_path)?;
        }
    }

    if args.in_place.is_some() {
        fs::write(file, &result)?;
    } else {
        print!("{}", result);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.expressions.is_empty() {
        eprintln!("sed: no expression given");
        std::process::exit(1);
    }

    if args.files.is_empty() {
        process_stdin(&args)
    } else {
        for file in &args.files {
            process_file(&args, file)?;
        }
        Ok(())
    }
}
