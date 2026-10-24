mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("md5sum 0.1.0");
        return;
    }

    if args.tag && args.check {
        eprintln!("md5sum: the --tag option is meaningless when verifying checksums");
        eprintln!("Try 'md5sum --help' for more information.");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    if args.check {
        let mut mismatches = 0usize;
        let mut unreadable = 0usize;
        for path in &args.files {
            if let Err(e) = check_md5_file(path, &args, &mut mismatches, &mut unreadable) {
                if !args.status {
                    eprintln!("md5sum: {}: {}", path.display(), e);
                }
                unreadable += 1;
            }
        }
        if !args.status {
            if mismatches > 0 {
                eprintln!(
                    "md5sum: WARNING: {} computed checksum{} did NOT match",
                    mismatches,
                    if mismatches == 1 { "" } else { "s" }
                );
            }
            if unreadable > 0 {
                eprintln!(
                    "md5sum: WARNING: {} listed file{} could not be read",
                    unreadable,
                    if unreadable == 1 { "" } else { "s" }
                );
            }
        }
        if mismatches > 0 || unreadable > 0 {
            exit_code = 1;
        }
    } else {
        if args.files.is_empty() {
            let stdin = std::io::stdin();
            let hash = compute_md5(stdin);
            print_hash("-", &hash, args.tag);
        } else {
            for path in &args.files {
                let path_str = path.to_string_lossy();
                if path_str == "-" {
                    let stdin = std::io::stdin();
                    let hash = compute_md5(stdin);
                    print_hash("-", &hash, args.tag);
                } else {
                    match File::open(path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let hash = compute_md5(reader);
                            print_hash(&path_str, &hash, args.tag);
                        }
                        Err(e) => {
                            eprintln!("md5sum: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn print_hash(name: &str, hash: &str, tag: bool) {
    if tag {
        println!("MD5 ({name}) = {hash}");
    } else {
        println!("{hash} {name}");
    }
}

fn compute_md5<R: Read>(mut reader: R) -> String {
    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => context.consume(&buffer[..n]),
            Err(_) => break,
        }
    }

    let digest = context.compute();
    format!("{:x}", digest)
}

fn check_md5_file(
    path: &std::path::Path,
    args: &cli::Args,
    mismatches: &mut usize,
    unreadable: &mut usize,
) -> Result<(), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;
    // Windows checklists are often saved by Notepad with a UTF-8 BOM.
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    let mut valid_entries = 0usize;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let mut expected_hash = parts[0];
        let mut file_path = parts[1..].join(" ");

        valid_entries += 1;

        // GNU escapes the digest together with the file name when the name
        // contains backslashes or newlines: "\<hash> \<file>".
        if let Some(rest) = expected_hash.strip_prefix('\\') {
            expected_hash = rest;
        }
        // Binary indicator ("md5sum -b" output): "<hash> *<file>".
        if let Some(rest) = file_path.strip_prefix('*') {
            file_path = rest.to_string();
        } else if file_path.starts_with('\\') && file_path.len() > 1 {
            file_path = file_path[1..].to_string();
        }

        if file_path == "-" {
            let hash = compute_md5(std::io::stdin());
            if hash == expected_hash {
                if !args.quiet && !args.status {
                    println!("-: OK");
                }
            } else {
                if !args.status {
                    eprintln!("-: FAILED");
                }
                *mismatches += 1;
            }
            continue;
        }

        match File::open(&file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let actual_hash = compute_md5(reader);

                if actual_hash == expected_hash {
                    if !args.quiet && !args.status {
                        println!("{}: OK", file_path);
                    }
                } else {
                    if !args.status {
                        eprintln!("{}: FAILED", file_path);
                    }
                    *mismatches += 1;
                }
            }
            Err(e) => {
                if !args.status {
                    eprintln!("md5sum: {}: {}", file_path, e);
                }
                *unreadable += 1;
            }
        }
    }

    if valid_entries == 0 {
        // GNU md5sum rejects check files that contain no usable lines.
        return Err("no properly formatted MD5 checksum lines found".to_string());
    }
    Ok(())
}
