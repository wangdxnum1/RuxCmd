use clap::Parser;
use sha3::{Digest, Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::fs::File;
use std::io::{self, BufReader, Read};

/// Compute SHA-3 checksums — a Windows port of sha3sum.
#[derive(Parser, Debug)]
#[command(name = "sha3sum", version, about)]
pub struct Args {
    /// SHA-3 algorithm bits: 224, 256, 384, or 512
    #[arg(long = "bits", value_name = "BITS", default_value_t = 512)]
    pub bits: u32,

    /// Read from FILE instead of stdin
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if !matches!(args.bits, 224 | 256 | 384 | 512) {
        eprintln!(
            "sha3sum: invalid --bits value: {} (must be 224, 256, 384, or 512)",
            args.bits
        );
        std::process::exit(1);
    }
    let files = if args.files.is_empty() {
        vec!["-".to_string()]
    } else {
        args.files.clone()
    };

    let mut failed = false;
    for path in &files {
        let name = if *path == "-" {
            "-".to_string()
        } else {
            path.clone()
        };
        let result = if name == "-" {
            compute_hash(io::stdin(), args.bits)
        } else {
            File::open(path)
                .map(|f| compute_hash(BufReader::new(f), args.bits))
                .unwrap_or_else(|e| Err(io::Error::other(e.to_string())))
        };
        match result {
            Ok(hash) => println!("{}  {}", hash, name),
            Err(e) => {
                eprintln!("sha3sum: {}: {}", name, e);
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}

fn compute_hash<R: Read>(mut reader: R, bits: u32) -> io::Result<String> {
    let mut buffer = [0u8; 8192];
    let hash = match bits {
        224 => {
            let mut h = Sha3_224::new();
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                h.update(&buffer[..n]);
            }
            hex::encode(h.finalize())
        }
        256 => {
            let mut h = Sha3_256::new();
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                h.update(&buffer[..n]);
            }
            hex::encode(h.finalize())
        }
        384 => {
            let mut h = Sha3_384::new();
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                h.update(&buffer[..n]);
            }
            hex::encode(h.finalize())
        }
        _ => {
            let mut h = Sha3_512::new();
            loop {
                let n = reader.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                h.update(&buffer[..n]);
            }
            hex::encode(h.finalize())
        }
    };
    Ok(hash)
}
