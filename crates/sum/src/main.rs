use clap::Parser;
use std::io::Read;

/// Checksum and count the blocks in a file - a Windows port of Linux sum
/// with both the BSD (default, 1K blocks) and System V (-s, 512 blocks)
/// algorithms.
#[derive(Parser, Debug)]
#[command(name = "sum", version, about)]
pub struct Args {
    /// Use the System V sum algorithm, use 512-byte blocks
    #[arg(short = 's', long = "sysv")]
    pub sysv: bool,

    /// Use the BSD sum algorithm, use 1K blocks (default)
    #[arg(short = 'r', long = "bsd")]
    pub bsd: bool,

    /// Files to sum (default: standard input)
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let sysv = if args.sysv { true } else { args.bsd && false };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut failed = false;

    if args.files.is_empty() {
        if let Err(e) = sum_stdin(sysv, &mut out) {
            eprintln!("sum: error reading stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        for file in &args.files {
            if file == "-" {
                if let Err(e) = sum_stdin(sysv, &mut out) {
                    eprintln!("sum: error reading stdin: {}", e);
                    failed = true;
                }
                continue;
            }
            match std::fs::File::open(file) {
                Ok(mut handle) => {
                    if let Err(e) = sum_stream(&mut handle, sysv, Some(file), &mut out) {
                        eprintln!("sum: {}: {}", file, e);
                        failed = true;
                    }
                }
                Err(e) => {
                    eprintln!("sum: {}: {}", file, e);
                    failed = true;
                }
            }
        }
    }
    std::process::exit(if failed { 1 } else { 0 });
}

fn sum_stdin(sysv: bool, out: &mut impl std::io::Write) -> Result<(), std::io::Error> {
    sum_stream(&mut std::io::stdin().lock(), sysv, None, out)
}

fn sum_stream(
    reader: &mut impl Read,
    sysv: bool,
    name: Option<&str>,
    out: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut checksum: u16 = 0;
    let mut blocks: u64 = 0;
    let mut buffer = [0u8; 65536];
    let block = if sysv { 512u64 } else { 1024 };
    let mut carried: u64 = 0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        for &byte in &buffer[..n] {
            if sysv {
                checksum = checksum.wrapping_add(u16::from(byte));
            } else {
                let rotate = (checksum >> 1) | ((checksum & 1) << 15);
                checksum = rotate.wrapping_add(u16::from(byte));
            }
        }
        carried += n as u64;
        blocks += carried / block;
        carried %= block;
    }
    // A final partial block still counts as one block, like GNU sum.
    if carried > 0 {
        blocks += 1;
    }

    if sysv {
        match name {
            Some(n) => writeln!(out, "{} {} {}", checksum, blocks, n)?,
            None => writeln!(out, "{} {}", checksum, blocks)?,
        }
    } else {
        writeln!(out, "{:05} {:>5}", checksum, blocks)?;
    }
    Ok(())
}
