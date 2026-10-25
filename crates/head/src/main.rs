mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();
    let separator = if args.zero_terminated { 0 } else { b'\n' };

    // Usage error (exit 1): `-n` and `-c` cannot be combined.
    if args.lines.is_some() && args.bytes.is_some() {
        eprintln!("head: cannot specify both -n and -c");
        std::process::exit(1);
    }

    let lines = args.lines.unwrap_or(10).max(0) as usize;
    let bytes = args.bytes.map(|b| b.max(0) as usize);

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        // `-v` prints the header for standard input too (GNU name, like tail).
        if let Err(e) = head_reader(
            stdin.lock(),
            "standard input",
            lines,
            bytes,
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

            if let Err(e) = head_file(file, &file_name, lines, bytes, header, separator) {
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
    lines: usize,
    bytes: Option<usize>,
    show_header: bool,
    separator: u8,
) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    head_reader(reader, name, lines, bytes, show_header, separator)
}

fn head_reader<R: Read>(
    mut reader: R,
    name: &str,
    lines: usize,
    bytes: Option<usize>,
    show_header: bool,
    separator: u8,
) -> Result<(), String> {
    if show_header {
        println!("==> {} <==", name);
    }

    let mut stdout = std::io::stdout();

    if let Some(bytes) = bytes {
        let mut buf = vec![0u8; bytes];
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("cannot read: {}", e))?;
        stdout
            .write_all(&buf[..n])
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        // Byte-safe line mode: every line is echoed with its original
        // terminator, so the `\r` of a CRLF survives, a final line without a
        // newline is not padded, and invalid UTF-8 passes through (the
        // pre-fix `lines()` + `writeln!` rewrote all three).
        let mut reader = BufReader::new(reader);
        let mut raw: Vec<u8> = Vec::new();
        for _ in 0..lines {
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

    Ok(())
}
