mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("column 0.1.0");
        return;
    }

    let right_cols: Vec<usize> = match args.table_right.as_ref() {
        Some(s) if !s.is_empty() => {
            let mut v = Vec::new();
            for p in s.split(',') {
                match p.trim().parse::<usize>() {
                    Ok(x) if x > 0 => v.push(x - 1),
                    _ => {
                        eprintln!("column: invalid -R value '{}'", s);
                        std::process::exit(1);
                    }
                }
            }
            v
        }
        _ => Vec::new(),
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut exit_code = 0i32;

    let mut push_lines = |r: Result<Box<dyn BufRead>, String>| {
        let reader = match r {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                exit_code = 1;
                return;
            }
        };
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("column: read error: {}", e);
                    exit_code = 1;
                    return;
                }
            };
            let parts: Vec<String> = if args.table {
                split_by_any(&line, &args.separator)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                line.split_whitespace().map(|s| s.to_string()).collect()
            };
            rows.push(parts);
        }
    };

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        push_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                push_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
            } else {
                match File::open(p) {
                    Ok(f) => push_lines(Ok(Box::new(BufReader::new(f)))),
                    Err(e) => push_lines(Err(format!("column: {}: {}", p.display(), e))),
                }
            }
        }
    }

    if rows.is_empty() {
        std::process::exit(exit_code);
    }

    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths: Vec<usize> = vec![0; ncols];
    for r in &rows {
        for (i, c) in r.iter().enumerate() {
            let w = display_width(c);
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (ridx, r) in rows.iter().enumerate() {
        if let Some(h) = args.header {
            if ridx > 0 && ridx % h == 0 {
                let header_row = rows[0].clone();
                let _ = write_row(
                    &mut out,
                    &header_row,
                    &widths,
                    &args.output_separator,
                    &right_cols,
                );
                let _ = out.write_all(b"\n");
            }
        }
        let _ = write_row(&mut out, r, &widths, &args.output_separator, &right_cols);
        let _ = out.write_all(b"\n");
    }

    std::process::exit(exit_code);
}

fn split_by_any<'a>(line: &'a str, seps: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let sep_chars: Vec<char> = seps.chars().collect();
    let mut start = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if sep_chars.contains(&c) {
            out.push(line[start..byte_pos(&chars, i)].into());
            i += 1;
            start = byte_pos(&chars, i);
        } else {
            i += 1;
        }
    }
    out.push(line[start..].into());
    out
}

fn byte_pos(chars: &[char], idx: usize) -> usize {
    chars[..idx.min(chars.len())]
        .iter()
        .map(|c| c.len_utf8())
        .sum()
}

fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| if (c as u32) < 0x80 { 1 } else { 2 })
        .sum()
}

fn write_row<W: Write>(
    out: &mut W,
    r: &[String],
    widths: &[usize],
    outsep: &str,
    right: &[usize],
) -> Result<(), std::io::Error> {
    let n = r.len();
    for (i, cell) in r.iter().enumerate() {
        let w = *widths.get(i).unwrap_or(&0);
        let dw = display_width(cell);
        let pad = if dw < w { w - dw } else { 0 };
        let is_right = right.contains(&i);
        if is_right {
            for _ in 0..pad {
                out.write_all(b" ")?;
            }
            out.write_all(cell.as_bytes())?;
        } else {
            out.write_all(cell.as_bytes())?;
            for _ in 0..pad {
                out.write_all(b" ")?;
            }
        }
        if i + 1 < n {
            out.write_all(outsep.as_bytes())?;
        }
    }
    Ok(())
}
