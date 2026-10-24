mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::cmp::Ordering;
use std::fs;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();

    let separator = parse_separator(&args);
    let key = args.key.as_deref().map(parse_key_spec);

    let mut lines: Vec<Line> = Vec::new();
    // GNU sort exits 2 when an operand cannot be read, while still
    // processing and printing the remaining input.
    let mut failures = 0usize;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if !read_lines(stdin.lock(), &mut lines) {
            failures += 1;
        }
    } else {
        for file in &args.files {
            // `-` means standard input, like GNU sort.
            if file.as_os_str() == "-" {
                let stdin = std::io::stdin();
                if !read_lines(stdin.lock(), &mut lines) {
                    failures += 1;
                }
                continue;
            }
            let file_name = file.display().to_string();
            match fs::File::open(file) {
                Ok(f) => {
                    if !read_lines(BufReader::new(f), &mut lines) {
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
        lines.sort_by(|a, b| compare_lines(&a.text, &b.text, key, separator, &args));

        if args.unique {
            // Deduplicate after sorting: two lines are duplicates when their
            // sort keys compare equal (with -k the key, not the whole line,
            // decides).
            lines.dedup_by(|a, b| {
                compare_keys(&a.text, &b.text, key, separator, &args) == Ordering::Equal
            });
        }

        if args.reverse {
            lines.reverse();
        }

        let mut stdout = std::io::stdout().lock();
        for line in &lines {
            // Echo the original bytes (invalid UTF-8 included); the line
            // terminator is always rebuilt as `\n`.
            let _ = stdout.write_all(&line.raw);
            let _ = stdout.write_all(b"\n");
        }
        let _ = stdout.flush();
    }

    if failures > 0 {
        std::process::exit(2);
    }
}

/// One input line: the original bytes with the line terminator removed, plus
/// a lossy UTF-8 view used for key extraction and comparison, so lines that
/// are not valid UTF-8 are sorted and echoed instead of being truncated.
struct Line {
    raw: Vec<u8>,
    text: String,
}

impl Line {
    fn new(raw: Vec<u8>) -> Self {
        let text = String::from_utf8_lossy(&raw).into_owned();
        Line { raw, text }
    }
}

/// Reads lines as raw bytes (matching happens on the lossy UTF-8 view, output
/// echoes the original bytes). Only the `\n` line terminator — and a `\r`
/// that is part of a `\r\n` — is removed; other trailing whitespace is part
/// of the line. Returns false when a read error interrupted the input
/// (previously `read_line(...).unwrap_or(0)` silently ended the input there).
fn read_lines<R: BufRead>(reader: R, lines: &mut Vec<Line>) -> bool {
    for line in ByteLines::new(reader) {
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
fn field_spans(line: &str, separator: Option<char>) -> Vec<(usize, usize)> {
    match separator {
        Some(sep) => {
            let mut spans = Vec::new();
            let mut start = 0;
            for (idx, c) in line.char_indices() {
                if c == sep {
                    spans.push((start, idx));
                    start = idx + c.len_utf8();
                }
            }
            spans.push((start, line.len()));
            spans
        }
        None => {
            let mut spans = Vec::new();
            let bytes = line.as_bytes();
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
    b == b' ' || b == b'\t'
}

/// The comparison key for `line`: the whole line, or the substring spanning
/// the selected fields. Fields beyond the end of the line are empty, and an
/// end field beyond the line extends the key to the end of the line.
fn extract_key(line: &str, key: Option<KeySpec>, separator: Option<char>) -> &str {
    let Some(spec) = key else {
        return line;
    };
    let spans = field_spans(line, separator);
    if spec.start > spans.len() {
        return "";
    }
    let start = spans[spec.start - 1].0;
    let end = match spec.end {
        Some(m) if m - 1 < spans.len() => spans[m - 1].1,
        _ => line.len(),
    };
    &line[start..end]
}

/// Primary comparison: only the sort keys decide (used for `-u` equality).
fn compare_keys(
    a: &str,
    b: &str,
    key: Option<KeySpec>,
    separator: Option<char>,
    args: &cli::Args,
) -> Ordering {
    let ka = extract_key(a, key, separator);
    let kb = extract_key(b, key, separator);
    if args.numeric {
        leading_number(ka)
            .partial_cmp(&leading_number(kb))
            .unwrap_or(Ordering::Equal)
    } else if args.human_numeric {
        human_number(ka)
            .partial_cmp(&human_number(kb))
            .unwrap_or(Ordering::Equal)
    } else if args.ignore_case {
        ka.to_lowercase().cmp(&kb.to_lowercase())
    } else {
        ka.cmp(kb)
    }
}

/// Full comparison used for ordering (sort and `-c`): falls back to a
/// whole-line byte comparison when keys tie, unless `-u` (GNU last-resort
/// comparison, disabled by -u/-s).
fn compare_lines(
    a: &str,
    b: &str,
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
        let cmp = compare_lines(&w[0].text, &w[1].text, key, separator, args);
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
    scan_leading_number(s.trim_start_matches([' ', '\t'])).0
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
    let trimmed = s.trim_start_matches([' ', '\t']);
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
