mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

/// A parsed `-n`/`-c` argument. The sign of the number decides between GNU's
/// two modes: `K` (or `+K`) prints the first K records, while `-K` prints all
/// but the last K records. `-0` is `-0`, not `0`, so the sign is parsed from
/// the string and never lost.
#[derive(Clone, Copy)]
enum Count {
    First(usize),
    AllButLast(usize),
}

impl Count {
    /// Parses `raw` following GNU coreutils: a leading `-` switches to
    /// all-but-last mode; anything that is not a plain digit string fails
    /// with `invalid number of lines|bytes: '<raw>'` (GNU wording, exit 1).
    fn parse(kind: &str, raw: &str) -> Result<Count, String> {
        let body = raw.strip_prefix('-').unwrap_or(raw);
        if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("invalid number of {}: '{}'", kind, raw));
        }
        let n = body
            .parse::<usize>()
            .map_err(|_| format!("invalid number of {}: '{}'", kind, raw))?;
        if raw.starts_with('-') {
            Ok(Count::AllButLast(n))
        } else {
            Ok(Count::First(n))
        }
    }
}

/// Which of `-n`/`-c` governs the output. GNU head does not reject the
/// combination: the option appearing LAST on the command line wins
/// (`head -n 5 -c 3` prints 3 bytes). With neither given, GNU defaults
/// to the first 10 lines.
#[derive(Clone, Copy)]
enum Selection {
    None,
    Lines(Count),
    Bytes(Count),
}

fn main() {
    let args = cli::Args::parse();
    let separator = if args.zero_terminated { 0 } else { b'\n' };

    let mut fail = |error: String| {
        eprintln!("head: {error}");
        std::process::exit(1);
    };

    // GNU converts numbers while its getopt loop consumes the options, so
    // even a losing option's invalid number fails, and when both are invalid
    // the FIRST-occurring option is reported: parse in occurrence order.
    let lines_occurrence = cli::lines_occurrence();
    let bytes_occurrence = cli::bytes_occurrence();
    let mut parse_count = |kind: &'static str, raw: &Option<String>| -> Option<Count> {
        raw.as_ref()
            .map(|raw| Count::parse(kind, raw))
            .transpose()
            .unwrap_or_else(&mut fail)
    };
    let (lines, bytes) = if lines_occurrence <= bytes_occurrence {
        let lines = parse_count("lines", &args.lines);
        let bytes = parse_count("bytes", &args.bytes);
        (lines, bytes)
    } else {
        let bytes = parse_count("bytes", &args.bytes);
        let lines = parse_count("lines", &args.lines);
        (lines, bytes)
    };

    let selection = match (lines, bytes) {
        (Some(l), Some(b)) => {
            if lines_occurrence > bytes_occurrence {
                Selection::Lines(l)
            } else {
                Selection::Bytes(b)
            }
        }
        (Some(l), None) => Selection::Lines(l),
        (None, Some(b)) => Selection::Bytes(b),
        (None, None) => Selection::None,
    };

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        // `-v` prints the header for standard input too (GNU name, like tail).
        if let Err(e) = head_reader(
            stdin.lock(),
            "standard input",
            selection,
            args.verbose,
            separator,
        ) {
            eprintln!("head: {}", e);
            std::process::exit(1);
        }
    } else {
        let multiple_files = args.files.len() > 1;
        // The per-file index does not influence the decision: -v forces the
        // header for every operand, -q suppresses it for all.
        let show_header = |_i: usize| -> bool {
            if args.verbose {
                true
            } else if args.quiet {
                false
            } else {
                multiple_files
            }
        };

        let mut exit_code = 0i32;

        for (i, file) in args.files.iter().enumerate() {
            let file_name = file.display().to_string();
            let header = show_header(i);

            if let Err(e) = head_file(file, &file_name, selection, header, separator) {
                eprintln!("head: {}: {}", file_name, e);
                exit_code = 1;
            }

            if i < args.files.len() - 1 && show_header(i + 1) {
                println!();
            }
        }

        std::process::exit(exit_code);
    }
}

fn head_file(
    path: &Path,
    name: &str,
    selection: Selection,
    show_header: bool,
    separator: u8,
) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    head_reader(reader, name, selection, show_header, separator)
}

fn head_reader<R: Read>(
    mut reader: R,
    name: &str,
    selection: Selection,
    show_header: bool,
    separator: u8,
) -> Result<(), String> {
    if show_header {
        println!("==> {} <==", name);
    }

    let mut stdout = std::io::stdout();

    if let Selection::Bytes(bytes) = selection {
        match bytes {
            Count::First(n) => {
                // `take` reads exactly n bytes (or all of them at EOF) without
                // pre-allocating a buffer of that size.
                let mut limited = Vec::new();
                reader
                    .take(n as u64)
                    .read_to_end(&mut limited)
                    .map_err(|e| format!("cannot read: {}", e))?;
                stdout
                    .write_all(&limited)
                    .map_err(|e| format!("cannot write: {}", e))?;
            }
            Count::AllButLast(n) => {
                // Negative byte count: everything except the last n bytes.
                let mut all = Vec::new();
                reader
                    .read_to_end(&mut all)
                    .map_err(|e| format!("cannot read: {}", e))?;
                let keep = all.len().saturating_sub(n);
                stdout
                    .write_all(&all[..keep])
                    .map_err(|e| format!("cannot write: {}", e))?;
            }
        }
        return Ok(());
    }

    let count = if let Selection::Lines(count) = selection {
        count
    } else {
        // Neither -n nor -c was given: GNU prints the first 10 lines.
        Count::First(10)
    };
    match count {
        Count::First(n) => {
            // Byte-safe line mode: every line is echoed with its original
            // terminator, so the `\r` of a CRLF survives, a final line without
            // a newline is not padded, and invalid UTF-8 passes through (the
            // pre-fix `lines()` + `writeln!` rewrote all three).
            let mut reader = BufReader::new(reader);
            let mut raw: Vec<u8> = Vec::new();
            for _ in 0..n {
                raw.clear();
                match reader.read_until(separator, &mut raw) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) => return Err(format!("cannot read: {}", e)),
                }
                stdout
                    .write_all(&raw)
                    .map_err(|e| format!("cannot write: {}", e))?;
            }
        }
        Count::AllButLast(n) => {
            // Negative line count: collect the records, then emit everything
            // except the last n of them. GNU does the same buffering for
            // negative counts on both files and standard input.
            let mut reader = BufReader::new(reader);
            let mut chunks: Vec<Vec<u8>> = Vec::new();
            loop {
                let mut raw = Vec::new();
                match reader.read_until(separator, &mut raw) {
                    Ok(0) => break,
                    Ok(_) => chunks.push(raw),
                    Err(e) => return Err(format!("cannot read: {}", e)),
                }
            }
            let keep = chunks.len().saturating_sub(n);
            for chunk in &chunks[..keep] {
                stdout
                    .write_all(chunk)
                    .map_err(|e| format!("cannot write: {}", e))?;
            }
        }
    }

    Ok(())
}
