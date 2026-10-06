mod cli;

use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = head_reader(stdin.lock(), "<stdin>", &args, false) {
            eprintln!("head: {}", e);
            std::process::exit(1);
        }
    } else {
        let multiple_files = args.files.len() > 1;
        let show_header = |i: usize| -> bool {
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

            if let Err(e) = head_file(file, &file_name, &args, header) {
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

fn head_file(path: &Path, name: &str, args: &cli::Args, show_header: bool) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    head_reader(reader, name, args, show_header)
}

fn head_reader<R: Read>(
    mut reader: R,
    name: &str,
    args: &cli::Args,
    show_header: bool,
) -> Result<(), String> {
    if show_header {
        println!("==> {} <==", name);
    }

    if let Some(bytes) = args.bytes {
        let bytes = bytes.max(0) as usize;
        let mut buf = vec![0u8; bytes];
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("cannot read: {}", e))?;
        std::io::stdout()
            .write_all(&buf[..n])
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        let lines = args.lines.max(0) as usize;
        let reader = BufReader::new(reader);
        let mut stdout = std::io::stdout();

        for (i, line) in reader.lines().enumerate() {
            if i >= lines {
                break;
            }
            let line = line.map_err(|e| format!("cannot read line: {}", e))?;
            writeln!(stdout, "{}", line).map_err(|e| format!("cannot write: {}", e))?;
        }
    }

    Ok(())
}
