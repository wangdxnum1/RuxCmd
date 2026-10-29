//! GNU echo semantics: leading option clusters are scanned by hand, so
//! `--` is an operand, unknown clusters like `-x` print verbatim, and only
//! --help/--version are special-cased like every coreutils tool.

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    let mut no_newline = false;
    let mut enable_escapes = false;
    let mut disable_escapes = false;
    let mut index = 0usize;

    while index < argv.len() {
        let arg = &argv[index];
        if !arg.starts_with('-') || arg == "-" {
            break;
        }
        match arg.as_str() {
            "--version" => {
                println!("echo {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" => {
                println!("Usage: echo [SHORT-OPTION]... [STRING]...");
                println!("  or:  echo LONG-OPTION");
                println!("Echo the STRING(s) to standard output.");
                println!();
                println!("  -n        do not output the trailing newline");
                println!("  -e        enable interpretation of backslash escapes");
                println!("  -E        disable interpretation of backslash escapes (default)");
                println!("      --help    display this help and exit");
                println!("      --version output version information and exit");
                return;
            }
            _ => {
                let mut cluster_valid = true;
                for ch in arg[1..].chars() {
                    match ch {
                        'n' => no_newline = true,
                        'e' => enable_escapes = true,
                        'E' => disable_escapes = true,
                        _ => {
                            cluster_valid = false;
                            break;
                        }
                    }
                }
                if !cluster_valid {
                    break;
                }
            }
        }
        index += 1;
    }

    let operands = &argv[index..];
    let text = operands.join(" ");

    let mut out: Vec<u8> = Vec::new();
    if enable_escapes && !disable_escapes {
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if !expand_escape(&mut chars, &mut out) {
                    // \c: produce no further output and no trailing newline.
                    write_stdout(&out);
                    return;
                }
            } else {
                let mut b = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
            }
        }
    } else {
        out.extend_from_slice(text.as_bytes());
    }

    if !no_newline {
        out.push(b'\n');
    }
    write_stdout(&out);
}

fn write_stdout(bytes: &[u8]) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(bytes);
    let _ = lock.flush();
}

/// Expand one backslash escape; returns false when \c stops all output.
fn expand_escape(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    out: &mut Vec<u8>,
) -> bool {
    match chars.next() {
        Some('a') => out.push(0x07),
        Some('b') => out.push(0x08),
        Some('e') => out.push(0x1B),
        Some('f') => out.push(0x0C),
        Some('n') => out.push(b'\n'),
        Some('r') => out.push(b'\r'),
        Some('t') => out.push(b'\t'),
        Some('v') => out.push(0x0B),
        Some('\\') => out.push(b'\\'),
        Some('"') => out.push(b'"'),
        Some('\'') => out.push(b'\''),
        Some('?') => out.push(b'?'),
        Some('c') => return false,
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
        Some(d) if ('0'..='7').contains(&d) => {
            // coreutils echo -e: the first octal digit plus up to three more.
            let mut value: u32 = d.to_digit(8).unwrap();
            let mut count = 0;
            while count < 3 {
                match chars.peek().and_then(|c| c.to_digit(8)) {
                    Some(next) => {
                        value = value * 8 + next;
                        chars.next();
                        count += 1;
                    }
                    None => break,
                }
            }
            out.push((value & 0xFF) as u8);
        }
        Some(other) => {
            out.push(b'\\');
            let mut b = [0u8; 4];
            out.extend_from_slice(other.encode_utf8(&mut b).as_bytes());
        }
        None => out.push(b'\\'),
    }
    true
}
