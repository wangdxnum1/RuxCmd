mod cli;

use clap::Parser;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

fn main() {
    let args = cli::Args::parse();

    // `-i` is accepted for script compatibility; there is no signal-ignore
    // mechanic wired up on Windows, so it is a documented no-op (see help).

    let mut files: Vec<std::fs::File> = Vec::new();

    for path in &args.files {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .append(args.append)
            .truncate(!args.append)
            .open(path)
        {
            Ok(file) => files.push(file),
            Err(e) => {
                eprintln!("tee: {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    // Byte-transparent pump (P0 #24): GNU tee never decodes or splits
    // lines, so CRLF, NUL bytes, invalid UTF-8, and a missing final
    // newline all pass through unchanged.
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut buf = [0u8; 8192];
    let mut exit_code = 0;

    loop {
        match stdin.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = stdout.write_all(&buf[..n]) {
                    eprintln!("tee: error writing to stdout: {}", e);
                    exit_code = 1;
                }
                for file in &mut files {
                    if let Err(e) = file.write_all(&buf[..n]) {
                        eprintln!("tee: error writing to file: {}", e);
                        exit_code = 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("tee: error reading from stdin: {}", e);
                exit_code = 1;
                break;
            }
        }
    }

    if let Err(e) = stdout.flush() {
        eprintln!("tee: error flushing stdout: {}", e);
        exit_code = 1;
    }
    for file in &mut files {
        if let Err(e) = file.flush() {
            eprintln!("tee: error flushing file: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}
