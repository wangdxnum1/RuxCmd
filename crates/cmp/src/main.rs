mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{Read, Write};
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("cmp 0.1.0");
        process::exit(0);
    }

    if cli.silent && cli.verbose {
        eprintln!("cmp: options -l and -s are incompatible");
        eprintln!("cmp: Try 'cmp --help' for more information.");
        process::exit(2);
    }

    // GNU cmp compares against '-' (standard input) when only one operand
    // is given; two operands are the normal case.
    let (file1, file2) = match (&cli.file1, &cli.file2) {
        (Some(f1), Some(f2)) => (f1.clone(), f2.clone()),
        (Some(f1), None) => (f1.clone(), "-".to_string()),
        (None, _) => {
            eprintln!("cmp: missing operand");
            eprintln!("cmp: Try 'cmp --help' for more information.");
            process::exit(2);
        }
    };

    let mut reader1 = open_input(&file1);
    let mut reader2 = open_input(&file2);

    let mut buf1 = [0u8; 8192];
    let mut buf2 = [0u8; 8192];
    let mut pending1: Vec<u8> = Vec::new();
    let mut pending2: Vec<u8> = Vec::new();
    let mut eof1 = false;
    let mut eof2 = false;
    let mut pos: u64 = 0;
    let mut newlines: u64 = 0;
    let mut last_was_newline = false;
    let mut any_diff = false;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    loop {
        if pending1.is_empty() && !eof1 {
            let n = fill(reader1.as_mut(), &mut buf1, &file1);
            if n == 0 {
                eof1 = true;
            } else {
                pending1.extend_from_slice(&buf1[..n]);
            }
        }
        if pending2.is_empty() && !eof2 {
            let n = fill(reader2.as_mut(), &mut buf2, &file2);
            if n == 0 {
                eof2 = true;
            } else {
                pending2.extend_from_slice(&buf2[..n]);
            }
        }

        let common = pending1.len().min(pending2.len());
        for i in 0..common {
            let b1 = pending1[i];
            let b2 = pending2[i];
            if b1 != b2 {
                any_diff = true;
                if cli.silent {
                    process::exit(1);
                }
                if cli.verbose {
                    let _ = writeln!(out, "{} {:o} {:o}", pos + 1, b1, b2);
                } else {
                    let _ = writeln!(
                        out,
                        "{} {} differ: char {}, line {}",
                        file1,
                        file2,
                        pos + 1,
                        newlines + 1
                    );
                    process::exit(1);
                }
            }
            last_was_newline = b1 == b'\n';
            pos += 1;
            if last_was_newline {
                newlines += 1;
            }
        }
        pending1.drain(..common);
        pending2.drain(..common);

        let report_eof = |name: &str, empty: bool| {
            if !cli.silent {
                if empty {
                    eprintln!("cmp: EOF on '{}' which is empty", name);
                } else if last_was_newline {
                    eprintln!(
                        "cmp: EOF on '{}' after byte {}, line {}",
                        name, pos, newlines
                    );
                } else {
                    eprintln!(
                        "cmp: EOF on '{}' after byte {}, in line {}",
                        name,
                        pos,
                        newlines + 1
                    );
                }
            }
            process::exit(1);
        };

        if pending1.is_empty() && pending2.is_empty() {
            if eof1 && eof2 {
                process::exit(if any_diff { 1 } else { 0 });
            }
            if eof1 {
                report_eof(&file1, pos == 0);
            }
            if eof2 {
                report_eof(&file2, pos == 0);
            }
        } else if pending1.is_empty() && eof1 {
            report_eof(&file1, pos == 0);
        } else if pending2.is_empty() && eof2 {
            report_eof(&file2, pos == 0);
        }
    }
}

fn open_input(name: &str) -> Box<dyn Read> {
    if name == "-" {
        Box::new(std::io::stdin())
    } else {
        match File::open(name) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("cmp: {}: {}", name, e);
                process::exit(2);
            }
        }
    }
}

/// Read at least one byte (up to the buffer capacity) so a shorter file
/// is observed as EOF while the other still has data; a mid-stream read
/// error exits 2 like other trouble in GNU cmp.
fn fill(reader: &mut dyn Read, buf: &mut [u8], name: &str) -> usize {
    let mut n = 0;
    while n == 0 {
        match reader.read(buf) {
            Ok(0) => return 0,
            Ok(k) => n = k,
            Err(e) => {
                eprintln!("cmp: {}: {}", name, e);
                process::exit(2);
            }
        }
    }
    n
}
