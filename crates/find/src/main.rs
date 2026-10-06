mod cli;

use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args = cli::Args::parse();

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    let re = args
        .regex
        .as_ref()
        .map(|r| Regex::new(r).expect("invalid regex"));

    for path in &paths {
        find(path, &args, &re, 0);
    }
}

fn find(path: &Path, args: &cli::Args, re: &Option<Regex>, depth: usize) {
    let mindepth = args.mindepth.unwrap_or(0);
    let maxdepth = args.maxdepth.unwrap_or(usize::MAX);

    if depth < mindepth || depth >= maxdepth {
        return;
    }

    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("find: {}: {}", path.display(), e);
            return;
        }
    };

    let is_dir = metadata.is_dir();
    let is_file = metadata.is_file();

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
            Err(e) => eprintln!("find: {}: {}", path.display(), e),
        }
    }
}

fn glob_match(name: &str, pattern: &str) -> bool {
    let pattern = pattern
        .replace(".", "\\.")
        .replace("*", ".*")
        .replace("?", ".");
    let re = Regex::new(&format!("^{}$", pattern)).unwrap();
    re.is_match(name)
}
