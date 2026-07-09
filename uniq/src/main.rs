mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("uniq 0.1.0");
        process::exit(0);
    }

    let reader: Box<dyn BufRead> = match &cli.file {
        Some(path) => {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("Error: cannot open file '{}': {}", path, e);
                process::exit(1);
            });
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(std::io::stdin())),
    };

    process_lines(reader, &cli);
}

fn process_lines(reader: Box<dyn BufRead>, cli: &Cli) {
    let mut lines = reader.lines();
    let mut prev_line: Option<String> = None;
    let mut count: usize = 0;

    while let Some(line_result) = lines.next() {
        let line = line_result.unwrap_or_else(|e| {
            eprintln!("Error reading line: {}", e);
            process::exit(1);
        });

        let key = make_key(&line, cli);

        if prev_line.is_none() {
            prev_line = Some(key);
            count = 1;
            continue;
        }

        let prev_key = prev_line.as_ref().unwrap();

        if key == *prev_key {
            count += 1;
        } else {
            print_line(&prev_line.unwrap(), count, cli);
            prev_line = Some(key);
            count = 1;
        }
    }

    if let Some(line) = prev_line {
        print_line(&line, count, cli);
    }
}

fn make_key(line: &str, cli: &Cli) -> String {
    let mut key = line.to_string();

    if let Some(n) = cli.skip_fields {
        let mut fields = key.split_whitespace();
        for _ in 0..n {
            fields.next();
        }
        key = fields.collect::<Vec<_>>().join(" ");
    }

    if cli.ignore_case {
        key = key.to_lowercase();
    }

    key
}

fn print_line(line: &str, count: usize, cli: &Cli) {
    if cli.duplicate_only && count == 1 {
        return;
    }

    if cli.unique_only && count > 1 {
        return;
    }

    if cli.count {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}