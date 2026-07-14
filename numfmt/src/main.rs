mod cli;

use clap::Parser;
use cli::{From, Round, To};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use std::cell::Cell;

const SI_SUFFIXES: [char; 8] = [' ', 'K', 'M', 'G', 'T', 'P', 'E', 'Z'];
const IEC_SUFFIXES: [char; 8] = SI_SUFFIXES;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("numfmt 0.1.0");
        return;
    }

    let suffix = args.suffix_opt.clone().unwrap_or_default();
    let is_file = !args.rest.is_empty() && std::path::Path::new(&args.rest[0]).exists();
    let exit_code = Cell::new(0i32);

    let mut process = |line: &str, is_header: bool| {
        if is_header {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", line);
            return;
        }
        let pieces: Vec<&str> = split_fields(line, &args.delimiter);
        let fidx = args.field.checked_sub(1).unwrap_or(0);
        if fidx >= pieces.len() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", line);
            return;
        }
        let token = pieces[fidx].trim();
        let value = match parse_numeric(token, args.from, args.from_unit, &suffix) {
            Some(v) => v,
            None => {
                eprintln!("numfmt: invalid number '{}'", token);
                exit_code.set(1);
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", line);
                return;
            }
        };
        let formatted = format_numeric(value, args.to, args.to_unit, args.round, &suffix, args.grouping);
        let mut joined = String::new();
        for (i, piece) in pieces.iter().enumerate() {
            if i > 0 { joined.push(' '); }
            if i == fidx {
                joined.push_str(&formatted);
            } else {
                joined.push_str(piece);
            }
        }
        let padded = apply_padding(&joined, args.padding);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", padded);
    };

    if args.rest.is_empty() || !is_file {
        let numbers: Vec<String> = args.rest.clone();
        if numbers.is_empty() {
            let stdin = std::io::stdin();
            for (i, line) in BufReader::new(stdin.lock()).lines().enumerate() {
                match line {
                    Ok(l) => process(&l, i < args.header),
                    Err(e) => { eprintln!("numfmt: stdin: {}", e); exit_code.set(1); }
                }
            }
        } else {
            for (i, n) in numbers.iter().enumerate() {
                process(n, i < args.header);
            }
        }
    } else {
        let fname = &args.rest[0];
        match File::open(fname) {
            Ok(f) => for (i, line) in BufReader::new(f).lines().enumerate() {
                match line {
                    Ok(l) => process(&l, i < args.header),
                    Err(e) => { eprintln!("numfmt: {}: {}", fname, e); exit_code.set(1); }
                }
            },
            Err(e) => { eprintln!("numfmt: {}: {}", fname, e); exit_code.set(1); }
        }
    }

    std::process::exit(exit_code.get());
}

fn split_fields<'a>(line: &'a str, delim: &str) -> Vec<&'a str> {
    let ds: Vec<char> = delim.chars().collect();
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut start = 0;
    let mut in_sep = true;
    for (i, c) in chars.iter().enumerate() {
        if ds.contains(c) {
            if !in_sep {
                out.push(slice_from_chars(line, &chars, start, i));
            }
            in_sep = true;
        } else {
            if in_sep { start = i; in_sep = false; }
        }
    }
    if !in_sep {
        out.push(slice_from_chars(line, &chars, start, chars.len()));
    }
    out
}

fn slice_from_chars<'a>(src: &'a str, chars: &[char], s: usize, e: usize) -> &'a str {
    let byte_s = chars[..s].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_e = chars[s..e.min(chars.len())].iter().map(|c| c.len_utf8()).sum::<usize>() + byte_s;
    &src[byte_s..byte_e.min(src.len())]
}

fn parse_numeric(s: &str, from: From, from_unit: f64, suffix: &str) -> Option<f64> {
    let s = s.strip_suffix(suffix).unwrap_or(s);
    let last = s.chars().last()?;
    let (mult, rest) = match from {
        From::None => {
            match s.parse::<f64>() {
                Ok(v) => return Some(v * from_unit),
                Err(_) => return None,
            }
        }
        From::Auto => parse_suffix(s, true),
        From::Si => parse_suffix(s, false),
        From::Iec => parse_suffix(s, true),
        From::IecI => parse_suffix(s, true),
    };
    let v: f64 = rest.parse().ok()?;
    Some(v * mult * from_unit)
}

fn parse_suffix(s: &str, binary: bool) -> (f64, &str) {
    let base: f64 = if binary { 1024.0 } else { 1000.0 };
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n == 0 { return (1.0, s); }
    let last = chars[n-1];
    if last.is_ascii_digit() || last == '.' || last == '-' {
        return (1.0, s);
    }
    let (suffix_char, skip) = if (last == 'i' || last == 'I') && n >= 2 {
        (chars[n-2].to_ascii_uppercase(), 2usize)
    } else {
        (last.to_ascii_uppercase(), 1usize)
    };
    let i = match suffix_char {
        'K' => 1, 'M' => 2, 'G' => 3, 'T' => 4, 'P' => 5, 'E' => 6, 'Z' => 7,
        _ => return (1.0, s),
    };
    let byte_end: usize = chars[..n-skip].iter().map(|c| c.len_utf8()).sum();
    (base.powi(i as i32), &s[..byte_end])
}

fn format_numeric(v: f64, to: To, to_unit: f64, rnd: Round, suffix: &str, grouping: bool) -> String {
    let value = v / to_unit;
    let base: f64 = match to {
        To::Si => 1000.0,
        To::Iec | To::IecI => 1024.0,
        To::Auto => if value >= 1024.0 { 1024.0 } else { 1000.0 },
        To::None => { return format_plain(value, grouping, 0, rnd, suffix); }
    };
    let mut idx = 0usize;
    let mut n = value;
    while idx + 1 < SI_SUFFIXES.len() && n.abs() >= base - 1e-9 {
        n /= base;
        idx += 1;
    }
    let s = if idx == 0 { String::new() } else {
        let ch = if matches!(to, To::Si) { SI_SUFFIXES[idx] } else { IEC_SUFFIXES[idx] };
        let ieci = if matches!(to, To::IecI) { "i" } else { "" };
        format!("{}{}", ch, ieci)
    };
    let precision = if idx == 0 { 0 } else { 1 };
    let num_str = format_plain(n, grouping, precision, rnd, "");
    format!("{}{}{}", num_str, s, suffix)
}

fn format_plain(v: f64, grouping: bool, decimals: usize, rnd: Round, suffix: &str) -> String {
    let rounded = apply_round(v, rnd, decimals);
    let mut out = if decimals == 0 {
        format!("{:.0}", rounded)
    } else {
        format!("{:.*}", decimals, rounded)
    };
    if grouping {
        out = add_grouping(&out);
    }
    out.push_str(suffix);
    out
}

fn apply_round(v: f64, r: Round, decimals: usize) -> f64 {
    let fact = 10f64.powi(decimals as i32);
    let scaled = v * fact;
    let done = match r {
        Round::Up => scaled.ceil(),
        Round::Down => scaled.floor(),
        Round::FromZero => if scaled >= 0.0 { scaled.ceil() } else { scaled.floor() },
        Round::TowardsZero => scaled.trunc(),
        Round::Nearest => scaled.round(),
    };
    done / fact
}

fn add_grouping(s: &str) -> String {
    let neg = s.starts_with('-');
    let body = if neg { &s[1..] } else { s };
    let dot_idx = body.find('.').unwrap_or(body.len());
    let intp = &body[..dot_idx];
    let mut chars: Vec<char> = intp.chars().collect();
    chars.reverse();
    let mut out: Vec<char> = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    out.reverse();
    let mut result: String = out.into_iter().collect();
    if dot_idx < body.len() {
        result.push_str(&body[dot_idx..]);
    }
    if neg { result.insert(0, '-'); }
    result
}

fn apply_padding(s: &str, padding: isize) -> String {
    if padding == 0 { return s.to_string(); }
    let n = padding.unsigned_abs() as usize;
    if s.len() >= n { return s.to_string(); }
    let pad = " ".repeat(n - s.len());
    if padding > 0 {
        format!("{}{}", s, pad)
    } else {
        format!("{}{}", pad, s)
    }
}
