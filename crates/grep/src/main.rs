mod cli;

use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    let mut pattern = args.pattern.clone();
    if args.line_regexp {
        pattern = format!("^{}$", pattern);
    }

    if args.ignore_case {
        pattern = format!("(?i){}", pattern);
    }

    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("grep: invalid regular expression: {}", e);
            std::process::exit(2);
        }
    };

    let multiple_files = args.files.len() > 1;
    let mut total_count = 0;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        total_count += grep_reader(stdin.lock(), "<stdin>", &re, &args);
        if args.count && !args.quiet {
            println!("{}", total_count);
        }
    } else {
        for file in &args.files {
            let file_name = file.display().to_string();
            if file.is_dir() {
                if args.recursive {
                    total_count += grep_directory(file, &re, &args, multiple_files);
                } else {
                    eprintln!("grep: {}: Is a directory", file_name);
                }
            } else {
                let count = match grep_file(file, &file_name, &re, &args, multiple_files) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("grep: {}: {}", file_name, e);
                        continue;
                    }
                };
                total_count += count;
            }
        }

        if args.count && !args.quiet && multiple_files {
            println!("{}", total_count);
        }
    }

    std::process::exit(if total_count > 0 { 0 } else { 1 });
}

fn grep_directory(dir: &Path, re: &Regex, args: &cli::Args, multiple_files: bool) -> u64 {
    let mut total_count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total_count += grep_directory(&path, re, args, multiple_files);
            } else {
                let file_name = path.display().to_string();
                if let Ok(count) = grep_file(&path, &file_name, re, args, true) {
                    total_count += count;
                }
            }
        }
    }

    total_count
}

fn grep_file(
    path: &Path,
    name: &str,
    re: &Regex,
    args: &cli::Args,
    _show_filename: bool,
) -> Result<u64, String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(grep_reader(reader, name, re, args))
}

fn grep_reader<R: BufRead>(mut reader: R, name: &str, re: &Regex, args: &cli::Args) -> u64 {
    let mut count = 0;
    let mut line_num = 0;

    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap_or(0) > 0 {
        line_num += 1;
        let line = buf.trim_end();
        let matched = re.is_match(line);
        let should_print = matched != args.invert_match;

        if should_print {
            count += 1;

            if !args.quiet && !args.count {
                if args.only_matching {
                    for cap in re.find_iter(line) {
                        if args.line_number {
                            println!("{}:{}:{}", name, line_num, cap.as_str());
                        } else if name != "<stdin>" {
                            println!("{}:{}", name, cap.as_str());
                        } else {
                            println!("{}", cap.as_str());
                        }
                    }
                } else {
                    if args.line_number {
                        println!("{}:{}:{}", name, line_num, line);
                    } else if name != "<stdin>" {
                        println!("{}:{}", name, line);
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }

        buf.clear();
    }

    count
}
