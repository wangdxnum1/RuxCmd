mod cli;

use clap::Parser;
use std::io::Write;

fn main() {
    let args = cli::Args::parse();
    let (out, had_error) = printf(&args.format, &args.args);

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(e) = lock.write_all(&out).and_then(|()| lock.flush()) {
        eprintln!("printf: write error: {}", e);
        std::process::exit(1);
    }
    if had_error {
        std::process::exit(1);
    }
}

fn printf(format: &str, args: &[String]) -> (Vec<u8>, bool) {
    let mut out: Vec<u8> = Vec::new();
    let mut had_error = false;
    let mut arg_index = 0usize;
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => expand_escape(&mut chars, &mut out),
            '%' => match chars.next() {
                Some('s') => {
                    if let Some(a) = args.get(arg_index) {
                        out.extend_from_slice(a.as_bytes());
                    }
                    arg_index += 1;
                }
                Some('d') => {
                    push_number(&mut out, args.get(arg_index), NumKind::Dec, &mut had_error);
                    arg_index += 1;
                }
                Some('x') => {
                    push_number(&mut out, args.get(arg_index), NumKind::Hex, &mut had_error);
                    arg_index += 1;
                }
                Some('f') => {
                    push_number(&mut out, args.get(arg_index), NumKind::Float, &mut had_error);
                    arg_index += 1;
                }
                Some('c') => {
                    if let Some(ch) = args.get(arg_index).and_then(|a| a.chars().next()) {
                        let mut buf = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                    }
                    arg_index += 1;
                }
                Some('%') => out.push(b'%'),
                Some(other) => {
                    out.push(b'%');
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(other.encode_utf8(&mut buf).as_bytes());
                }
                None => out.push(b'%'),
            },
            _ => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }

    (out, had_error)
}

enum NumKind {
    Dec,
    Hex,
    Float,
}

/// Format one numeric directive. A missing argument becomes 0; an
/// unconvertible argument reports an error, yields 0 and lets the process
/// exit 1 after the whole format is processed (GNU coreutils behavior).
fn push_number(out: &mut Vec<u8>, arg: Option<&String>, kind: NumKind, had_error: &mut bool) {
    let Some(a) = arg else {
        out.push(b'0');
        return;
    };
    let parsed = match kind {
        NumKind::Dec => a.trim_start().parse::<i64>().map(|n| n.to_string()).map_err(|_| ()),
        NumKind::Hex => {
            a.trim_start()
                .parse::<u64>()
                .map(|n| format!("{:x}", n))
                .map_err(|_| ())
        }
        NumKind::Float => a.trim_start().parse::<f64>().map(|n| n.to_string()).map_err(|_| ()),
    };
    match parsed {
        Ok(s) => out.extend_from_slice(s.as_bytes()),
        Err(_) => {
            eprintln!("printf: '{}': expected a numeric value", a);
            *had_error = true;
            out.push(b'0');
        }
    }
}

/// Expand one backslash escape (the echo -e family used by GNU printf):
/// \n \r \t \\ \" \0NNN (up to three octal digits) and \xHH (up to two hex
/// digits). Unknown escapes keep the backslash and the character verbatim.
fn expand_escape(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut Vec<u8>) {
    match chars.next() {
        Some('n') => out.push(b'\n'),
        Some('r') => out.push(b'\r'),
        Some('t') => out.push(b'\t'),
        Some('\\') => out.push(b'\\'),
        Some('"') => out.push(b'"'),
        Some('0') => {
            let mut value: u32 = 0;
            let mut digits = 0;
            while digits < 3 {
                match chars.peek().and_then(|c| c.to_digit(8)) {
                    Some(d) => {
                        value = value * 8 + d;
                        chars.next();
                        digits += 1;
                    }
                    None => break,
                }
            }
            out.push((value & 0xFF) as u8);
        }
        Some('x') => {
            let mut value: u32 = 0;
            let mut digits = 0;
            while digits < 2 {
                match chars.peek().and_then(|c| c.to_digit(16)) {
                    Some(d) => {
                        value = value * 16 + d;
                        chars.next();
                        digits += 1;
                    }
                    None => break,
                }
            }
            if digits == 0 {
                out.extend_from_slice(b"\\x");
            } else {
                out.push((value & 0xFF) as u8);
            }
        }
        Some(other) => {
            out.push(b'\\');
            let mut buf = [0u8; 4];
            out.extend_from_slice(other.encode_utf8(&mut buf).as_bytes());
        }
        None => out.push(b'\\'),
    }
}
