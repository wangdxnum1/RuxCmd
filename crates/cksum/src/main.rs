mod cli;

use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::OnceLock;

/// Generator polynomial of the POSIX cksum CRC (F(x) per POSIX, degree 32).
const CRC32_POLY: u32 = 0x04C1_1DB7;

fn main() {
    let args = cli::parse_args();

    // GNU 9.x cksum accepts several digests via -a; this port implements the
    // POSIX crc algorithm only.
    if !args.algorithm.eq_ignore_ascii_case("crc") {
        eprintln!("cksum: {} is not supported", args.algorithm);
        eprintln!("Try 'cksum --help' for more information.");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        match compute_cksum(std::io::stdin()) {
            Ok((crc, length)) => println!("{crc} {length}"),
            Err(error) => {
                eprintln!("cksum: -: {error}");
                exit_code = 1;
            }
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            let result = if path_str == "-" {
                compute_cksum(std::io::stdin())
            } else {
                File::open(path).and_then(|file| compute_cksum(BufReader::new(file)))
            };
            match result {
                Ok((crc, length)) => print_cksum(&path_str, crc, length),
                Err(error) => {
                    eprintln!("cksum: {}: {error}", path.display());
                    exit_code = 1;
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn print_cksum(name: &str, crc: u32, length: u64) {
    println!("{crc} {length} {name}");
}

/// Build the 256-entry MSB-first CRC table for the POSIX polynomial once.
fn crc32_table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (i, entry) in table.iter_mut().enumerate() {
            let mut crc = (i as u32) << 24;
            for _ in 0..8 {
                if crc & 0x8000_0000 != 0 {
                    crc = (crc << 1) ^ CRC32_POLY;
                } else {
                    crc <<= 1;
                }
            }
            *entry = crc;
        }
        table
    })
}

fn compute_cksum<R: Read>(mut reader: R) -> std::io::Result<(u32, u64)> {
    let table = crc32_table();
    let mut crc: u32 = 0;
    let mut length: u64 = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = match reader.read(&mut buffer) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if bytes_read == 0 {
            break;
        }
        length += bytes_read as u64;
        for &byte in &buffer[..bytes_read] {
            crc = (crc << 8) ^ table[(((crc >> 24) as u8) ^ byte) as usize];
        }
    }
    // POSIX extends the CRC with the length of the file, least significant
    // byte first (GNU processes only the significant bytes).
    let mut n = length;
    while n != 0 {
        crc = (crc << 8) ^ table[(((crc >> 24) as u8) ^ (n as u8)) as usize];
        n >>= 8;
    }
    Ok((!crc, length))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FailedReader;
    impl Read for FailedReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("injected failure"))
        }
    }
    #[test]
    fn read_failure_is_not_a_checksum() {
        assert!(compute_cksum(FailedReader).is_err());
    }
}
