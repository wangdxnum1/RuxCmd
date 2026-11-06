mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

// util-linux column aligns fill-mode columns on tab stops of 8 cells.
const TABCHAR_CELLS: usize = 8;

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

    // Lines are stored verbatim, with the trailing '\n' when present, so the
    // fill mode can keep the '\r' of CRLF input like GNU column, while the
    // table mode keeps its historical CRLF-stripping behavior.
    let mut lines: Vec<String> = Vec::new();
    let mut exit_code = 0i32;

    let mut push_lines = |r: Result<Box<dyn BufRead>, String>| {
        let mut reader = match r {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                exit_code = 1;
                return;
            }
        };
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => break,
                Ok(_) => lines.push(std::mem::take(&mut buf)),
                Err(e) => {
                    eprintln!("column: read error: {}", e);
                    exit_code = 1;
                    return;
                }
            }
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

    if lines.is_empty() {
        std::process::exit(exit_code);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if args.table {
        let rows: Vec<Vec<String>> = lines
            .iter()
            .map(|l| {
                // BufRead::lines() semantics: a trailing "\r\n" is stripped,
                // a lone '\r' without a newline is kept.
                let text = match l.strip_suffix('\n') {
                    Some(rest) => rest.strip_suffix('\r').unwrap_or(rest),
                    None => l.as_str(),
                };
                match args.separator.as_deref() {
                    Some(seps) => split_by_any(text, seps)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                    None => split_whitespace_runs(text)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                }
            })
            .collect();

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

        for (ridx, r) in rows.iter().enumerate() {
            if let Some(h) = args.header
                && ridx > 0
                && ridx % h == 0
            {
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
            let _ = write_row(&mut out, r, &widths, &args.output_separator, &right_cols);
            let _ = out.write_all(b"\n");
        }
    } else {
        fill_mode(&mut out, &lines, terminal_width());
    }

    std::process::exit(exit_code);
}

// GNU column without -t (util-linux 2.40.2, columnate_fillcols): every line is
// one entry, verbatim. Empty and whitespace-only lines are skipped. Entries are
// arranged into as many tab-aligned columns as the width allows, filling
// columns top-to-bottom before starting a new row. An entry at least as wide
// as the terminal switches GNU to "simple" mode: every entry on its own line.
fn fill_mode<W: Write>(out: &mut W, lines: &[String], termwidth: usize) {
    let entries: Vec<&str> = lines
        .iter()
        .filter_map(|l| {
            let l = l.strip_suffix('\n').unwrap_or(l.as_str());
            // GNU skips a line only when every character is whitespace
            // (skip_space + empty check); any other line is one entry.
            if l.bytes()
                .all(|b| matches!(b, b' ' | b'\t' | b'\r' | 0x0B | 0x0C))
            {
                None
            } else {
                Some(l)
            }
        })
        .collect();

    let maxw = entries
        .iter()
        .map(|e| fill_display_width(e))
        .max()
        .unwrap_or(0);
    if maxw >= termwidth {
        for e in entries {
            let _ = out.write_all(e.as_bytes());
            let _ = out.write_all(b"\n");
        }
        return;
    }

    // The longest entry's width is rounded up to the next multiple of 8 (a
    // width that is already a multiple of 8 advances one full tab stop).
    let maxlength = (maxw + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
    let numcols = (termwidth / maxlength).max(1);
    let numrows = entries.len().div_ceil(numcols);

    for row in 0..numrows {
        let mut chcnt: usize = 0;
        let mut endcol = maxlength;
        let mut base = row;
        for _ in 0..numcols {
            let _ = out.write_all(entries[base].as_bytes());
            chcnt += fill_display_width(entries[base]);
            base += numrows;
            if base >= entries.len() {
                break; // the last column is never padded
            }
            while ((chcnt + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1)) <= endcol {
                let _ = out.write_all(b"\t");
                chcnt = (chcnt + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
            }
            endcol += maxlength;
        }
        let _ = out.write_all(b"\n");
    }
}

// GNU column uses the size of the terminal on stdout when available, then the
// COLUMNS environment variable, then 80 (get_terminal_width(80)). Reading the
// Windows console size would need an extra win32 dependency, so COLUMNS is the
// only override here.
fn terminal_width() -> usize {
    if let Ok(v) = std::env::var("COLUMNS") {
        // util-linux get_env_int: strictly parsed, 0 < n <= INT_MAX.
        if let Ok(w) = v.trim().parse::<i64>()
            && w > 0
            && w <= i64::from(i32::MAX)
        {
            return w as usize;
        }
    }
    80
}

// GNU width() sums wcwidth() over the characters and ignores every character
// with a non-positive cell count (control characters, including '\t').
// Non-ASCII follows the project's table-mode convention (2 cells), which
// matches wcwidth for CJK text.
fn fill_display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let u = c as u32;
            if u < 0x20 || u == 0x7f {
                0
            } else if u < 0x80 {
                1
            } else {
                2
            }
        })
        .sum()
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

// GNU default for -t: a run of separator whitespace counts as a single
// delimiter, and leading/trailing runs produce no empty columns.
fn split_whitespace_runs(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut field_start: Option<usize> = None;
    for (i, c) in line.char_indices() {
        if c == ' ' || c == '\t' {
            if let Some(s) = field_start.take() {
                out.push(&line[s..i]);
            }
        } else if field_start.is_none() {
            field_start = Some(i);
        }
    }
    if let Some(s) = field_start {
        out.push(&line[s..]);
    }
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
        let pad = w.saturating_sub(dw);
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
