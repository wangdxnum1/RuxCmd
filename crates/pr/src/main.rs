mod cli;

use chrono::Local;
use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let argv: Vec<OsString> = std::env::args_os().collect();
    let args = cli::Args::parse_from(attach_bare_n_value(argv));

    if args.version {
        println!("pr 0.1.0");
        return;
    }

    let default_header = match args.files.first().cloned() {
        Some(p) => p.display().to_string(),
        None => String::from(""),
    };
    let header_text = args.header.clone().unwrap_or(default_header);

    let date_str = format_date();

    let mut all_lines: Vec<String> = Vec::new();
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = read_lines(&mut BufReader::new(stdin.lock()), &mut all_lines) {
            eprintln!("pr: stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = read_lines(&mut BufReader::new(stdin.lock()), &mut all_lines) {
                    eprintln!("pr: stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(p) {
                    Ok(f) => {
                        if let Err(e) = read_lines(&mut BufReader::new(f), &mut all_lines) {
                            eprintln!("pr: {}: {}", p.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("pr: {}: {}", p.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    let (num_sep, num_width) = match parse_number_spec(&args.number_lines) {
        Ok(v) => v,
        Err(spec) => {
            eprintln!("pr: invalid --number-lines argument '{}'", spec);
            std::process::exit(1);
        }
    };
    let indent_str: String = " ".repeat(args.indent);
    let page_len = if args.length <= 10 { 66 } else { args.length };
    let header_lines = if args.omit_header { 0 } else { 5 };
    let body_len = page_len - header_lines;

    let processed: Vec<String> = preprocess(&all_lines, &args);
    let page_width = args.page_width.max(1);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let pages = if processed.is_empty() {
        1
    } else {
        (processed.len() + body_len - 1) / body_len.max(1)
    };

    for pno in 0..pages {
        let start = pno * body_len;
        let end = std::cmp::min(start + body_len, processed.len());
        let body: &[String] = if start < end {
            &processed[start..end]
        } else {
            &[]
        };

        if !args.omit_header {
            let _ = write_header(
                &mut out,
                &indent_str,
                &header_text,
                &date_str,
                pno + 1,
                page_width,
            );
        }

        for (i, line) in body.iter().enumerate() {
            let mut out_line = String::new();
            out_line.push_str(&indent_str);
            if let Some(ref sep) = num_sep {
                let num = start + i + 1;
                out_line.push_str(&format!("{:>w$}{}", num, sep, w = num_width));
            }
            let padded = truncate_or_keep(line, page_width - (out_line.len() - indent_str.len()));
            out_line.push_str(&padded);
            if args.double_space {
                out_line.push('\n');
            }
            let _ = writeln!(out, "{}", out_line);
        }

        let trailer_len = body_len - body.len();
        if !args.omit_header && !args.form_feed {
            for _ in 0..trailer_len {
                let _ = writeln!(out, "{}", indent_str);
            }
        }

        if args.form_feed {
            let _ = out.write_all(b"\x0C");
        }
    }

    std::process::exit(exit_code);
}

fn format_date() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

// GNU pr's -n takes its spec attached (`-n5`, `-n:5`); a bare `-n` must not
// swallow the following file name, so force an explicit empty value for it.
fn attach_bare_n_value(argv: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(argv.len() + 1);
    for a in argv {
        let bare = a == OsStr::new("-n");
        out.push(a);
        if bare {
            out.push(OsString::from(""));
        }
    }
    out
}

// GNU syntax `-n[SEP][DIGITS]`: a non-digit first char is the separator,
// remaining digits the width; bare/empty spec means TAB and width 5.
fn parse_number_spec(spec: &Option<String>) -> Result<(Option<String>, usize), String> {
    let Some(s) = spec else {
        return Ok((None, 0));
    };
    if s.is_empty() {
        return Ok((Some(String::from("\t")), 5));
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    let (sep, rest) = if first.is_ascii_digit() {
        (String::from("\t"), s.clone())
    } else {
        (first.to_string(), chars.as_str().to_string())
    };
    if rest.is_empty() {
        return Ok((Some(sep), 5));
    }
    match rest.parse::<usize>() {
        Ok(w) => Ok((Some(sep), w.max(1))),
        Err(_) => Err(s.clone()),
    }
}

fn read_lines<R: BufRead>(r: &mut R, out: &mut Vec<String>) -> Result<(), std::io::Error> {
    for line in r.lines() {
        out.push(line?);
    }
    Ok(())
}

fn preprocess(lines: &[String], args: &cli::Args) -> Vec<String> {
    if args.two_column {
        let n = lines.len();
        let half = (n + 1) / 2;
        let col_w = (args.page_width / 2).saturating_sub(1);
        let mut out = Vec::with_capacity(half);
        for i in 0..half {
            let mut row = String::new();
            if i < lines.len() {
                let a = truncate_or_keep(&lines[i], col_w);
                row.push_str(&a);
                for _ in a.len()..col_w {
                    row.push(' ');
                }
            } else {
                for _ in 0..col_w {
                    row.push(' ');
                }
            }
            row.push(' ');
            let ri = half + i;
            if ri < lines.len() {
                let b = truncate_or_keep(&lines[ri], col_w);
                row.push_str(&b);
            }
            out.push(row);
        }
        out
    } else {
        lines.to_vec()
    }
}

fn truncate_or_keep(s: &str, w: usize) -> String {
    let mut result = String::with_capacity(w);
    let mut count = 0usize;
    for c in s.chars() {
        let cw = if (c as u32) < 0x80 { 1 } else { 2 };
        if count + cw > w {
            break;
        }
        result.push(c);
        count += cw;
    }
    result
}

fn write_header<W: Write>(
    out: &mut W,
    indent: &str,
    header: &str,
    date: &str,
    page_no: usize,
    page_w: usize,
) -> Result<(), std::io::Error> {
    let total_inner = page_w - indent.len();
    let page_text = format!("Page {}", page_no);
    let center_head = center(header, total_inner);
    let mut line_a = String::from(indent);
    line_a.push_str(&truncate_or_keep(date, total_inner.min(date.len())));
    for _ in line_a.len()..page_w {
        line_a.push(' ');
    }
    let pg_start = page_w.saturating_sub(page_text.len());
    line_a.truncate(pg_start);
    line_a.push_str(&page_text);
    writeln!(out, "{}", line_a)?;

    let center_line = format!("{}{}", indent, center_head);
    writeln!(out, "{}", center_line)?;

    let mut sep = String::from(indent);
    for _ in 0..total_inner {
        sep.push('=');
    }
    writeln!(out, "{}", sep)?;
    writeln!(out, "")?;
    writeln!(out, "")?;
    Ok(())
}

fn center(s: &str, w: usize) -> String {
    let n = s.len();
    if n >= w {
        return s.to_string();
    }
    let lpad = (w - n) / 2;
    let rpad = w - n - lpad;
    format!("{}{}{}", " ".repeat(lpad), s, " ".repeat(rpad))
}
