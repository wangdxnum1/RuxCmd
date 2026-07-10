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

    let mut exit_code = 0i32;

    if args.check {
        if args.files.is_empty() {
            eprintln!("md5sum: option --check requires FILE argument");
            exit_code = 1;
        } else {
            for path in &args.files {
                if let Err(e) = check_md5_file(path, args.quiet) {
                    eprintln!("md5sum: {}: {}", path.display(), e);
                    exit_code = 1;
                }
            }
        }
    } else {
        if args.files.is_empty() {
            let stdin = std::io::stdin();
            let hash = compute_md5(stdin);
            println!("{} -", hash);
        } else {
            for path in &args.files {
                let path_str = path.to_string_lossy();
                if path_str == "-" {
                    let stdin = std::io::stdin();
                    let hash = compute_md5(stdin);
                    println!("{} -", hash);
                } else {
                    match File::open(path) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            let hash = compute_md5(reader);
                            println!("{} {}", hash, path.display());
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

fn check_md5_file(path: &std::path::Path, quiet: bool) -> Result<(), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).map_err(|e| e.to_string())?;
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    let mut failed = 0;
    let mut _passed = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let expected_hash = parts[0];
        let file_path = parts[1..].join(" ");

        let file_path = if file_path.starts_with('\\') && file_path.len() > 1 {
            &file_path[1..]
        } else {
            file_path.as_str()
        };

        match File::open(file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let actual_hash = compute_md5(reader);

                if actual_hash == expected_hash {
                    if !quiet {
                        println!("{}: OK", file_path);
                    }
                    _passed += 1;
                } else {
                    eprintln!("{}: FAILED", file_path);
                    failed += 1;
                }
            }
            Err(e) => {
                eprintln!("md5sum: {}: {}", file_path, e);
                failed += 1;
            }
        }
    }

    if failed > 0 {
        Err(format!("{} computed checksum(s) did NOT match", failed))
    } else {
        Ok(())
    }
}