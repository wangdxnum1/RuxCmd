mod cli;

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::parse_args();

    let mut exit_code = 0i32;

    if args.check {
        for path in &args.files {
            if let Err(e) = check_hash_file(path, args.quiet) {
                eprintln!("sha256sum: {}: {}", path.display(), e);
                exit_code = 1;
            }
        }
    } else {
        if args.files.is_empty() {
            let stdin = std::io::stdin();
            let hash = compute_sha256(stdin);
            println!("{} -", hash);
        } else {
            for path in &args.files {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        let hash = compute_sha256(reader);
                        println!("{} {}", hash, path.display());
                    }
                    Err(e) => {
                        eprintln!("sha256sum: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn compute_sha256<R: Read>(mut reader: R) -> String {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer).unwrap_or(0);
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let result = hasher.finalize();
    hex::encode(result)
}

fn check_hash_file(path: &std::path::Path, quiet: bool) -> Result<(), std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let mut mismatches = 0;

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

        let file_path = if file_path == "-" {
            continue;
        } else {
            std::path::Path::new(&file_path)
        };

        match File::open(file_path) {
            Ok(f) => {
                let reader = BufReader::new(f);
                let actual_hash = compute_sha256(reader);

                if actual_hash == expected_hash {
                    if !quiet {
                        println!("{}: OK", file_path.display());
                    }
                } else {
                    eprintln!("{}: FAILED", file_path.display());
                    mismatches += 1;
                }
            }
            Err(e) => {
                eprintln!("sha256sum: {}: {}", file_path.display(), e);
                mismatches += 1;
            }
        }
    }

    if mismatches > 0 {
        std::process::exit(1);
    }

    Ok(())
}
