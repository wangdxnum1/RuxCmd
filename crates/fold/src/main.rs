mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("fold 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = BufReader::new(std::io::stdin());
        if let Err(e) = process_reader(stdin, &args) {
            eprintln!("fold: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = BufReader::new(std::io::stdin());
                if let Err(e) = process_reader(stdin, &args) {
                    eprintln!("fold: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_reader(reader, &args) {
                            eprintln!("fold: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("fold: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

/// Read lines as raw bytes (`read_until`), so non-UTF-8 input is folded
/// instead of aborting with an encoding error. Only the `\n` terminator is
/// removed; any `\r` stays part of the content, as before.
fn process_reader<R: BufRead>(mut reader: R, args: &cli::Args) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut raw: Vec<u8> = Vec::new();

    loop {
        raw.clear();
        let bytes_read = reader.read_until(b'\n', &mut raw)?;
        if bytes_read == 0 {
            break;
        }

        let has_newline = raw.last() == Some(&b'\n');
        if has_newline {
            raw.pop();
        }

        if args.bytes {
            // Pure byte slicing: segments are written back verbatim, so the
            // input bytes are conserved regardless of encoding.
            fold_bytes(&mut out, &raw, args.width, args.spaces)?;
        } else {
            // Character mode works on the lossy text (invalid bytes become
            // U+FFFD) so every character still counts as one column.
            let content = String::from_utf8_lossy(&raw);
            fold_chars(
                &mut out,
                &content.chars().collect::<Vec<char>>(),
                args.width,
                args.spaces,
            )?;
        }

        if has_newline {
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn fold_chars<W: Write>(
    out: &mut W,
    chars: &[char],
    width: usize,
    split_on_spaces: bool,
) -> Result<(), std::io::Error> {
    if width == 0 {
        let line: String = chars.iter().collect();
        return out.write_all(line.as_bytes());
    }

    let mut start = 0;
    while start < chars.len() {
        let mut end = start + width;
        if end > chars.len() {
            end = chars.len();
        }

        if split_on_spaces && end < chars.len() {
            let mut split_pos = end;
            for i in (start..end).rev() {
                if chars[i].is_whitespace() {
                    split_pos = i + 1;
                    break;
                }
            }
            if split_pos != end {
                end = split_pos;
            }
        }

        let segment: String = chars[start..end].iter().collect();
        out.write_all(segment.as_bytes())?;

        start = end;
        if start < chars.len() {
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn fold_bytes<W: Write>(
    out: &mut W,
    bytes: &[u8],
    width: usize,
    split_on_spaces: bool,
) -> Result<(), std::io::Error> {
    if width == 0 {
        return out.write_all(bytes);
    }

    let mut start = 0;
    while start < bytes.len() {
        let mut end = start + width;
        if end > bytes.len() {
            end = bytes.len();
        }

        if split_on_spaces && end < bytes.len() {
            let mut split_pos = end;
            for i in (start..end).rev() {
                if bytes[i].is_ascii_whitespace() {
                    split_pos = i + 1;
                    break;
                }
            }
            if split_pos != end {
                end = split_pos;
            }
        }

        out.write_all(&bytes[start..end])?;

        start = end;
        if start < bytes.len() {
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}
