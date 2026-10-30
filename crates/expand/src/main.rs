mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("expand 0.1.0");
        return;
    }

    let stops = match parse_tabs(&args.tabs) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("expand: {}", msg);
            std::process::exit(1);
        }
    };

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process(&mut stdin.lock(), &stops, args.initial) {
            eprintln!("expand: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                process(&mut stdin.lock(), &stops, args.initial)
            } else {
                match File::open(p) {
                    Ok(mut f) => process(&mut f, &stops, args.initial),
                    Err(e) => {
                        eprintln!("expand: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("expand: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

enum Stops {
    Periodic(usize),
    Explicit(Vec<usize>),
}

fn parse_tabs(s: &str) -> Result<Stops, String> {
    if !s.contains(',') && !s.chars().any(|c: char| c.is_whitespace()) {
        match s.parse::<usize>() {
            Ok(0) => return Err("tab size cannot be 0".to_string()),
            Ok(n) => return Ok(Stops::Periodic(n)),
            Err(_) => {}
        }
    }
    let mut v: Vec<usize> = Vec::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()) {
        if part.is_empty() {
            continue;
        }
        match part.parse::<usize>() {
            Ok(0) => return Err("tab size cannot be 0".to_string()),
            Ok(x) => {
                if let Some(&last) = v.last()
                    && x <= last
                {
                    return Err("tab sizes must be ascending".to_string());
                }
                v.push(x);
            }
            Err(_) => return Err(format!("invalid tabs value '{}'", s)),
        }
    }
    if v.is_empty() {
        Err(format!("invalid tabs value '{}'", s))
    } else {
        Ok(Stops::Explicit(v))
    }
}

fn next_stop(stops: &Stops, col: usize) -> usize {
    match stops {
        Stops::Periodic(n) => {
            if *n == 0 {
                col + 1
            } else {
                (col / n + 1) * n
            }
        }
        Stops::Explicit(v) => {
            for s in v {
                if *s > col {
                    return *s;
                }
            }
            col + 1
        }
    }
}

/// Expand tabs over raw bytes: CR bytes and invalid UTF-8 pass through
/// untouched, and an unterminated final line stays unterminated.
/// Columns count one per character (or per stray byte).
fn process<R: Read>(reader: &mut R, stops: &Stops, initial: bool) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut buf = [0u8; 16384];
    let mut carry: Vec<u8> = Vec::new();
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        carry.extend_from_slice(&buf[..n]);
        let mut start = 0;
        while let Some(pos) = carry[start..].iter().position(|&b| b == b'\n') {
            let end = start + pos + 1;
            expand_line(&carry[start..end], stops, initial, &mut out)?;
            start = end;
        }
        carry.drain(..start);
    }
    if !carry.is_empty() {
        expand_line(&carry, stops, initial, &mut out)?;
    }
    Ok(())
}

fn expand_line(
    line: &[u8],
    stops: &Stops,
    initial: bool,
    out: &mut impl Write,
) -> Result<(), std::io::Error> {
    let content = if line.last() == Some(&b'\n') {
        &line[..line.len() - 1]
    } else {
        line
    };
    let mut col = 0usize;
    let mut past_init = false;
    let mut i = 0usize;
    while i < content.len() {
        match content[i] {
            b'\t' => {
                if !past_init || !initial {
                    let ns = next_stop(stops, col);
                    let n = if ns > col { ns - col } else { 1 };
                    for _ in 0..n {
                        out.write_all(b" ")?;
                    }
                    col = ns;
                } else {
                    out.write_all(b"\t")?;
                    col += 1;
                }
                i += 1;
            }
            b' ' => {
                out.write_all(b" ")?;
                col += 1;
                i += 1;
            }
            first => {
                let len = utf8_char_len(first);
                if len > 1
                    && i + len <= content.len()
                    && let Ok(s) = std::str::from_utf8(&content[i..i + len])
                {
                    out.write_all(s.as_bytes())?;
                    col += 1;
                    i += len;
                } else {
                    out.write_all(&content[i..i + 1])?;
                    col += 1;
                    i += 1;
                }
                past_init = true;
            }
        }
    }
    if line.last() == Some(&b'\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn utf8_char_len(first: u8) -> usize {
    match first {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}
