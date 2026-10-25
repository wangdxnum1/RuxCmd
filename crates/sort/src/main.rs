mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::cmp::Ordering;
use std::fs;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();
    let record_separator = if args.zero_terminated { 0 } else { b'\n' };

    let separator = parse_separator(&args);
    let key = args.key.as_deref().map(parse_key_spec);

    let mut lines: Vec<Line> = Vec::new();
    // GNU sort exits 2 when an operand cannot be read, while still
    // processing and printing the remaining input.
    let mut failures = 0usize;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if !read_lines(stdin.lock(), &mut lines, record_separator) {
            failures += 1;
        }
    } else {
        for file in &args.files {
            // `-` means standard input, like GNU sort.
            if file.as_os_str() == "-" {
                let stdin = std::io::stdin();
                if !read_lines(stdin.lock(), &mut lines, record_separator) {
                    failures += 1;
                }
                continue;
            }
            let file_name = file.display().to_string();
            match fs::File::open(file) {
                Ok(f) => {
                    if !read_lines(BufReader::new(f), &mut lines, record_separator) {
                        failures += 1;
                    }
                }
                Err(e) => {
                    eprintln!("sort: {}: {}", file_name, e);
                    failures += 1;
                }
            }
        }
    }

    if args.check {
        if !is_sorted(&lines, key, separator, &args) {
            eprintln!("sort: disorder detected");
            std::process::exit(1);
        }
    } else {
        lines.sort_by(|a, b| compare_lines(&a.raw, &b.raw, key, separator, &args));

        if args.unique {
            // Deduplicate after sorting: two lines are duplicates when their
            // sort keys compare equal (with -k the key, not the whole line,
            // decides).
            lines.dedup_by(|a, b| {
                compare_keys(&a.raw, &b.raw, key, separator, &args) == Ordering::Equal
            });
        }

        if args.reverse {
            lines.reverse();
        }

        let mut stdout = std::io::stdout().lock();
        for line in &lines {
            // Echo the original bytes and rebuild the selected terminator.
            let _ = stdout.write_all(&line.raw);
            let _ = stdout.write_all(&[record_separator]);
        }
        let _ = stdout.flush();
    }

    if failures > 0 {
        std::process::exit(2);
    }
}

/// One input record, with its original bytes and no record terminator.
struct Line {
    raw: Vec<u8>,
}

impl Line {
    fn new(raw: Vec<u8>) -> Self {
        Line { raw }
    }
}

/// Read raw records. LF mode retains historical CRLF stripping; NUL mode
/// removes only the NUL. A read error fails rather than truncating silently.
fn read_lines<R: BufRead>(reader: R, lines: &mut Vec<Line>, separator: u8) -> bool {
    for line in ByteLines::with_separator(reader, separator) {
        match line {
            Ok(raw) => lines.push(Line::new(raw)),
            Err(e) => {
                eprintln!("sort: read error: {}", e);
                return false;
            }
        }
    }
    true
}

/// `-t` accepts exactly one character; anything else is a usage error (exit 1).
fn parse_separator(args: &cli::Args) -> Option<char> {
    let spec = args.delimiter.as_deref()?;
    let mut chars = spec.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(c),
        (None, _) => {
            eprintln!("sort: empty field separator");
            std::process::exit(1);
        }
        _ => {
            eprintln!("sort: multi-character field separator '{}'", spec);
            std::process::exit(1);
        }
    }
}

/// Minimal GNU `-k F1[,F2]` key: fields are 1-indexed; the key spans field
/// F1 through field F2 (inclusive), or through end of line when F2 is
/// omitted. Field-character offsets (`-k2.3`) and per-key modifiers
/// (`-k2n`) are not supported.
#[derive(Debug, Clone, Copy)]
struct KeySpec {
    start: usize,
    end: Option<usize>,
}

fn parse_key_spec(spec: &str) -> KeySpec {
    fn invalid(spec: &str) -> ! {
        eprintln!("sort: invalid field specification '{}'", spec);
        std::process::exit(1);
    }

    let (start_str, end_str) = match spec.split_once(',') {
        Some((s, e)) => (s, Some(e)),
        None => (spec, None),
    };
    let start: usize = start_str.trim().parse().unwrap_or_else(|_| invalid(spec));
    let end = end_str.map(|e| e.trim().parse::<usize>().unwrap_or_else(|_| invalid(spec)));
    if start == 0 || end.is_some_and(|e| e == 0 || e < start) {
        invalid(spec);
    }
    KeySpec { start, end }
}

/// Byte spans (start, end) of the fields in `line`.
///
/// With a separator: fields are the runs between separator occurrences, and
/// a leading separator creates an empty first field (GNU `-t` semantics).
///
/// Without a separator: fields are the whitespace-separated tokens, and the
/// run of blanks preceding a token is part of that field (GNU behavior),
/// so a field's span starts at the first blank of the preceding run.
fn field_spans(line: &[u8], separator: Option<char>) -> Vec<(usize, usize)> {
    match separator {
        Some(sep) => {
            let mut spans = Vec::new();
            let mut start = 0;
            let mut encoded = [0; 4];
            let sep = sep.encode_utf8(&mut encoded).as_bytes();
            let mut idx = 0;
            while idx + sep.len() <= line.len() {
                if &line[idx..idx + sep.len()] == sep {
                    spans.push((start, idx));
                    idx += sep.len();
                    start = idx;
                } else {
                    idx += 1;
                }
            }
            spans.push((start, line.len()));
            spans
        }
        None => {
            let mut spans = Vec::new();
            let bytes = line;
            let mut i = 0;
            while i < bytes.len() {
                let run_start = i;
                while i < bytes.len() && is_blank(bytes[i]) {
                    i += 1;
                }
                if i == bytes.len() {
                    break;
                }
                while i < bytes.len() && !is_blank(bytes[i]) {
                    i += 1;
                }
                spans.push((run_start, i));
            }
            spans
        }
    }
}

fn is_blank(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n'
}

/// The comparison key for `line`: the whole line, or the substring spanning
/// the selected fields. Fields beyond the end of the line are empty, and an
/// end field beyond the line extends the key to the end of the line.
fn extract_key(line: &[u8], key: Option<KeySpec>, separator: Option<char>) -> &[u8] {
    let Some(spec) = key else {
        return line;
    };
    let spans = field_spans(line, separator);
    if spec.start > spans.len() {
        return b"";
    }
    let start = spans[spec.start - 1].0;
    let end = match spec.end {
        Some(m) if m - 1 < spans.len() => spans[m - 1].1,
        _ => line.len(),
    };
    &line[start..end]
}

fn fold_key(bytes: &[u8]) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_lowercase().into_bytes(),
        Err(_) => bytes.iter().map(u8::to_ascii_lowercase).collect(),
    }
}
/// Primary comparison: only the sort keys decide (used for `-u` equality).
fn compare_keys(
    a: &[u8],
    b: &[u8],
    key: Option<KeySpec>,
    separator: Option<char>,
    args: &cli::Args,
) -> Ordering {
    let ka = extract_key(a, key, separator);
    let kb = extract_key(b, key, separator);
    if args.numeric {
        leading_number(&String::from_utf8_lossy(ka))
            .partial_cmp(&leading_number(&String::from_utf8_lossy(kb)))
            .unwrap_or(Ordering::Equal)
    } else if args.human_numeric {
        human_number(&String::from_utf8_lossy(ka))
            .partial_cmp(&human_number(&String::from_utf8_lossy(kb)))
            .unwrap_or(Ordering::Equal)
    } else if args.ignore_case {
        fold_key(ka).cmp(&fold_key(kb))
    } else {
        ka.cmp(kb)
    }
}

/// Full comparison used for ordering (sort and `-c`): falls back to a
/// whole-line byte comparison when keys tie, unless `-u` (GNU last-resort
/// comparison, disabled by -u/-s).
fn compare_lines(
    a: &[u8],
    b: &[u8],
    key: Option<KeySpec>,
    separator: Option<char>,
    args: &cli::Args,
) -> Ordering {
    let ord = compare_keys(a, b, key, separator, args);
    if ord == Ordering::Equal && !args.unique {
        return a.cmp(b);
    }
    ord
}

fn is_sorted(
    lines: &[Line],
    key: Option<KeySpec>,
    separator: Option<char>,
    args: &cli::Args,
) -> bool {
    lines.windows(2).all(|w| {
        let cmp = compare_lines(&w[0].raw, &w[1].raw, key, separator, args);
        if args.reverse {
            // Reverse-sorted means non-increasing.
            cmp != Ordering::Less
        } else {
            cmp != Ordering::Greater
        }
    })
}

/// The leading number of `s` (optional +/- sign, digits, optional decimal
/// fraction) as used by GNU `-n`. Strings without a leading number are 0.
fn leading_number(s: &str) -> f64 {
    scan_leading_number(s.trim_start_matches([' ', '\t', '\n'])).0
}

/// Scans the leading number in `s`; returns its value and the byte offset
/// just past it, or (0.0, 0) when `s` does not start with a number.
fn scan_leading_number(s: &str) -> (f64, usize) {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - int_start;
    let mut frac_digits = 0;
    if i < bytes.len() && bytes[i] == b'.' {
        let frac_start = i + 1;
        let mut j = frac_start;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        frac_digits = j - frac_start;
        if frac_digits > 0 {
            i = j;
        }
    }
    if int_digits == 0 && frac_digits == 0 {
        return (0.0, 0);
    }
    (s[..i].parse::<f64>().unwrap_or(0.0), i)
}

/// Human-readable number as used by GNU `-h`: a leading number followed by
/// an optional K/M/G/T/P/E/Z/Y suffix, 1024-based. Strings without a
/// leading number are 0.
fn human_number(s: &str) -> f64 {
    let trimmed = s.trim_start_matches([' ', '\t', '\n']);
    let (value, end) = scan_leading_number(trimmed);
    if end == 0 {
        return 0.0;
    }
    let rest = trimmed[end..].trim_start_matches([' ', '\t']);
    let multiplier = match rest.chars().next().map(|c| c.to_ascii_lowercase()) {
        Some('k') => 1024.0,
        Some('m') => 1024f64.powi(2),
        Some('g') => 1024f64.powi(3),
        Some('t') => 1024f64.powi(4),
        Some('p') => 1024f64.powi(5),
        Some('e') => 1024f64.powi(6),
        Some('z') => 1024f64.powi(7),
        Some('y') => 1024f64.powi(8),
        _ => 1.0,
    };
    value * multiplier
}
