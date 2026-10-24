mod cli;

use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // GNU find spells its long options with a single dash (-name, -type,
    // -maxdepth, ...); rewrite them to clap's -- form before parsing.
    let argv: Vec<String> = std::env::args()
        .map(|a| {
            let gnu_long = [
                "-name",
                "-type",
                "-maxdepth",
                "-mindepth",
                "-regex",
                "-basename",
                "-ignore-case",
            ];
            if gnu_long.contains(&a.as_str()) {
                format!("-{}", a)
            } else {
                a
            }
        })
        .collect();
    let args = cli::Args::parse_from(argv);

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    let re = args.regex.as_ref().map(|r| match Regex::new(r) {
        Ok(re) => re,
        Err(e) => {
            eprintln!("find: invalid regex '{}': {}", r, e);
            std::process::exit(1);
        }
    });

    for path in &paths {
        find(path, &args, &re, 0);
    }

    if HAD_ERROR.with(|c| c.get()) {
        std::process::exit(1);
    }
}

thread_local! {
    static HAD_ERROR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn find(path: &Path, args: &cli::Args, re: &Option<Regex>, depth: usize) {
    let mindepth = args.mindepth.unwrap_or(0);
    let maxdepth = args.maxdepth.unwrap_or(usize::MAX);

    if depth < mindepth || depth > maxdepth {
        return;
    }

    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("find: {}: {}", path.display(), e);
            HAD_ERROR.with(|c| c.set(true));
            return;
        }
    };

    let is_dir = metadata.is_dir();
    let is_file = metadata.is_file();

    if let Some(c) = args.file_type {
        if c != 'd' && c != 'f' {
            eprintln!("find: invalid argument '{}' to '-type'", c);
            std::process::exit(1);
        }
    }

    let matches_type = match args.file_type {
        Some('d') => is_dir,
        Some('f') => is_file,
        None => true,
        _ => true,
    };

    if matches_type {
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        let matches_name = args.name.as_ref().map_or(true, |pattern| {
            let target = if args.ignore_case {
                name.to_lowercase()
            } else {
                name.to_string()
            };
            let pattern = if args.ignore_case {
                pattern.to_lowercase()
            } else {
                pattern.clone()
            };
            match pattern.contains('*') || pattern.contains('?') {
                true => glob_match(&target, &pattern),
                false => target == pattern,
            }
        });

        let matches_regex = re.as_ref().map_or(true, |r| r.is_match(&name));

        if matches_name && matches_regex {
            if args.basename {
                println!("{}", name);
            } else {
                println!("{}", path.display());
            }
        }
    }

    if is_dir && depth < maxdepth {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    find(&entry.path(), args, re, depth + 1);
                }
            }
            Err(e) => {
                eprintln!("find: {}: {}", path.display(), e);
                HAD_ERROR.with(|c| c.set(true));
            }
        }
    }
}

fn glob_match(name: &str, pattern: &str) -> bool {
    // Translate glob wildcards and escape every other regex metacharacter,
    // so a malformed pattern can no longer panic the regex engine.
    let mut regex = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '(' | ')' | '[' | ']' | '{' | '}' | '+' | '^' | '$' | '|' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }
    regex.push('$');
    match Regex::new(&regex) {
        Ok(re) => re.is_match(name),
        Err(_) => false,
    }
}
