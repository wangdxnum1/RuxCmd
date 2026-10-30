mod cli;

use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use std::fs::File;
use std::io::{self, BufReader, Read};

fn main() {
    let args = cli::parse_args();
    if args.tag && args.check {
        eprintln!("b2sum: the --tag option is meaningless when verifying checksums");
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
                compute_hash(io::stdin(), args.length)
            } else {
                File::open(&path).and_then(|f| compute_hash(BufReader::new(f), args.length))
            };
            hash.map(|hash| print_hash(&name, &hash, args.tag))
        };
        if let Err(error) = result {
            if !args.status {
                eprintln!("b2sum: {name}: {error}");
            }
            failures += 1;
        }
    }
    if args.check && !args.status {
        if mismatches > 0 {
            eprintln!(
                "b2sum: WARNING: {} computed checksum{} did NOT match",
                mismatches,
                if mismatches == 1 { "" } else { "s" }
            );
        }
        if failures > 0 {
            eprintln!(
                "b2sum: WARNING: {} listed file{} could not be read",
                failures,
                if failures == 1 { "" } else { "s" }
            );
        }
    }
    if failures + mismatches > 0 {
        std::process::exit(1);
    }
}

fn print_hash(name: &str, hash: &str, tag: bool) {
    if tag {
        println!("BLAKE2b-512 ({name}) = {hash}");
    } else {
        println!("{hash}  {name}");
    }
}

fn compute_hash<R: Read>(mut reader: R, length_bits: u32) -> io::Result<String> {
    let bytes = (length_bits / 8) as usize;
    let mut hasher = Blake2bVar::new(bytes).map_err(|_| invalid_length())?;
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
    let mut output = vec![0u8; bytes];
    hasher
        .finalize_variable(&mut output)
        .map_err(|_| invalid_length())?;
    Ok(hex::encode(output))
}

fn invalid_length() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "invalid digest length")
}

/// Parse one manifest line: `[BLAKE2b-512 (name) = digest]` or the
/// two-space (or space-star binary) form. The digest length follows the
/// configured --length.
fn parse_checksum(line: &str, digest_chars: usize) -> Option<(String, String)> {
    let (escaped, line) = match line.strip_prefix('\\') {
        Some(line) => (true, line),
        None => (false, line),
    };
    let (digest, filename) = if let Some(tagged) = line.strip_prefix("BLAKE2b-512 (") {
        let (filename, digest) = tagged.rsplit_once(") = ")?;
        (digest, filename)
    } else {
        let (digest, rest) = line.split_once(' ')?;
        (digest, rest.strip_prefix([' ', '*']).unwrap_or(rest))
    };
    if digest.len() != digest_chars
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        || filename.is_empty()
    {
        return None;
    }
    if !escaped {
        return Some((digest.to_string(), filename.to_string()));
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
    Some((digest.to_string(), name))
}

fn check_hash_reader<R: Read>(
    mut reader: R,
    args: &cli::Args,
    mismatches: &mut usize,
    failures: &mut usize,
) -> io::Result<()> {
    let digest_chars = (args.length / 8) * 2;
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    let mut valid_entries = 0;
    for line in content.lines() {
        let Some((expected, filename)) = parse_checksum(line, digest_chars as usize) else {
            continue;
        };
        valid_entries += 1;
        let actual = if filename == "-" {
            compute_hash(io::stdin(), args.length)
        } else {
            File::open(&filename).and_then(|f| compute_hash(BufReader::new(f), args.length))
        };
        match actual {
            Ok(actual) if actual.eq_ignore_ascii_case(&expected) => {
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
                    eprintln!("b2sum: {filename}: {error}");
                    println!("{filename}: FAILED open or read");
                }
                *failures += 1;
            }
        }
    }
    if valid_entries == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "no properly formatted checksum lines found",
        ));
    }
    Ok(())
}
