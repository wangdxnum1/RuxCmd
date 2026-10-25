mod cli;

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, Read};

fn main() {
    let args = cli::parse_args();
    if args.zero && args.check {
        eprintln!("sha256sum: the --zero option is not supported when verifying checksums");
        std::process::exit(1);
    }
    if args.tag && args.check {
        eprintln!("sha256sum: the --tag option is meaningless when verifying checksums");
        std::process::exit(1);
    }
    let files = if args.files.is_empty() {
        vec![std::path::PathBuf::from("-")]
    } else {
        args.files.clone()
    };
    let mut failures = 0;
    let mut mismatches = 0;
    for path in files {
        let name = path.to_string_lossy();
        let result = if args.check {
            let reader: io::Result<Box<dyn Read>> = if name == "-" {
                Ok(Box::new(io::stdin()))
            } else {
                File::open(&path).map(|f| Box::new(BufReader::new(f)) as Box<dyn Read>)
            };
            reader
                .and_then(|reader| check_hash_reader(reader, &args, &mut mismatches, &mut failures))
        } else {
            let hash = if name == "-" {
                compute_hash(io::stdin())
            } else {
                File::open(&path).and_then(|f| compute_hash(BufReader::new(f)))
            };
            hash.map(|hash| print_hash(&name, &hash, args.tag, args.zero))
        };
        if let Err(error) = result {
            if !args.status {
                eprintln!("sha256sum: {name}: {error}");
            }
            failures += 1;
        }
    }
    if args.check && !args.status {
        if mismatches > 0 {
            eprintln!(
                "sha256sum: WARNING: {} computed checksum{} did NOT match",
                mismatches,
                if mismatches == 1 { "" } else { "s" }
            );
        }
        if failures > 0 {
            eprintln!(
                "sha256sum: WARNING: {} listed file{} could not be read",
                failures,
                if failures == 1 { "" } else { "s" }
            );
        }
    }
    if failures + mismatches > 0 {
        std::process::exit(1);
    }
}

fn escaped_name(name: &str) -> String {
    name.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn print_hash(name: &str, hash: &str, tag: bool, zero: bool) {
    let escaped = !zero && name.contains(['\\', '\n', '\r']);
    let name = if escaped {
        escaped_name(name)
    } else {
        name.to_string()
    };
    let end = if zero { '\0' } else { '\n' };
    let prefix = if escaped { "\\" } else { "" };
    if tag {
        print!("{prefix}SHA256 ({name}) = {hash}{end}");
    } else {
        print!("{prefix}{hash}  {name}{end}");
    }
}

fn compute_hash<R: Read>(mut reader: R) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = match reader.read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn parse_checksum(line: &str) -> Option<(&str, String)> {
    let (escaped, line) = match line.strip_prefix('\\') {
        Some(line) => (true, line),
        None => (false, line),
    };
    let (digest, filename) = if let Some(tagged) = line.strip_prefix("SHA256 (") {
        let (filename, digest) = tagged.rsplit_once(") = ")?;
        (digest, filename)
    } else {
        let (digest, rest) = line.split_once(' ')?;
        // Accept the historical single-space form too, preserving the rest
        // verbatim; two-space text and space-star binary are GNU formats.
        (digest, rest.strip_prefix([' ', '*']).unwrap_or(rest))
    };
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) || filename.is_empty() {
        return None;
    }
    if !escaped {
        return Some((digest, filename.to_string()));
    }
    let mut name = String::new();
    let mut chars = filename.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            name.push(ch);
            continue;
        }
        name.push(match chars.next()? {
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            _ => return None,
        });
    }
    Some((digest, name))
}

fn check_hash_reader<R: Read>(
    mut reader: R,
    args: &cli::Args,
    mismatches: &mut usize,
    failures: &mut usize,
) -> io::Result<()> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    let mut valid_entries = 0;
    for line in content.lines() {
        let Some((expected, filename)) = parse_checksum(line) else {
            continue;
        };
        valid_entries += 1;
        let actual = if filename == "-" {
            compute_hash(io::stdin())
        } else {
            File::open(&filename).and_then(|f| compute_hash(BufReader::new(f)))
        };
        match actual {
            Ok(actual) if actual.eq_ignore_ascii_case(expected) => {
                if !args.quiet && !args.status {
                    println!("{filename}: OK");
                }
            }
            Ok(_) => {
                if !args.status {
                    println!("{filename}: FAILED");
                }
                *mismatches += 1;
            }
            Err(error) => {
                if !args.status {
                    eprintln!("sha256sum: {filename}: {error}");
                    println!("{filename}: FAILED open or read");
                }
                *failures += 1;
            }
        }
    }
    if valid_entries == 0 {
        return Err(io::Error::other(
            "no properly formatted SHA256 checksum lines found",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FailsAfterData(bool);
    impl Read for FailsAfterData {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.0 {
                return Err(io::Error::other("injected read error"));
            }
            self.0 = true;
            buffer[0] = b'x';
            Ok(1)
        }
    }
    #[test]
    fn partial_read_errors_do_not_yield_a_digest() {
        assert!(compute_hash(FailsAfterData(false)).is_err());
    }
}
