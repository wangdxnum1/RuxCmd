mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("unexpand 0.1.0");
        return;
    }

    let n = if args.tabs == 0 { 8 } else { args.tabs };

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process(BufReader::new(stdin.lock()), n, args.all) {
            eprintln!("unexpand: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                process(BufReader::new(stdin.lock()), n, args.all)
            } else {
                match File::open(p) {
                    Ok(f) => process(BufReader::new(f), n, args.all),
                    Err(e) => {
                        eprintln!("unexpand: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("unexpand: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

fn process<R: BufRead>(reader: R, n: usize, all: bool) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in reader.lines() {
        let line = line?;
        let mut col = 0usize;
        let mut space_run = 0usize;
        let mut in_leading = true;
        for ch in line.chars() {
            match ch {
                ' ' => {
                    if in_leading || all {
                        space_run += 1;
                        let next_col = col + 1;
                        if next_col % n == 0 && space_run >= 1 {
                            out.write_all(b"\t")?;
                            col = next_col;
                            space_run = 0;
                        } else {
                            col = next_col;
                        }
                    } else {
                        flush_spaces(&mut out, &mut space_run)?;
                        out.write_all(b" ")?;
                        col += 1;
                    }
                }
                '\t' => {
                    flush_spaces(&mut out, &mut space_run)?;
                    out.write_all(b"\t")?;
                    let ns = ((col / n) + 1) * n;
                    col = ns;
                }
                c => {
                    in_leading = false;
                    flush_spaces(&mut out, &mut space_run)?;
                    let mut b = [0u8; 4];
                    let s = c.encode_utf8(&mut b);
                    out.write_all(s.as_bytes())?;
                    col += 1;
                }
            }
        }
        flush_spaces(&mut out, &mut space_run)?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn flush_spaces<W: Write>(out: &mut W, n: &mut usize) -> Result<(), std::io::Error> {
    for _ in 0..*n {
        out.write_all(b" ")?;
    }
    *n = 0;
    Ok(())
}
