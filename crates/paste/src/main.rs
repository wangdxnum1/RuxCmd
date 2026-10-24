mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    // GNU paste: `-d LIST` uses the list's characters in turn between output
    // columns, restarting from the first character once the list is exhausted.
    // An explicitly empty list means no separator; without `-d` it is TAB.
    let delimiters: Vec<char> = args
        .delimiters
        .map(|d| d.chars().collect())
        .unwrap_or_else(|| vec!['\t']);
    let serial = args.serial;

    let mut exit_code = 0i32;

    let readers: Vec<Box<dyn BufRead>> = if args.files.is_empty() {
        vec![Box::new(BufReader::new(std::io::stdin()))]
    } else {
        args.files
            .iter()
            .map(|path| {
                let path_str = path.to_string_lossy();
                if path_str == "-" {
                    Box::new(BufReader::new(std::io::stdin())) as Box<dyn BufRead>
                } else {
                    match File::open(path) {
                        Ok(file) => Box::new(BufReader::new(file)) as Box<dyn BufRead>,
                        Err(e) => {
                            eprintln!("paste: {}: {}", path.display(), e);
                            exit_code = 1;
                            Box::new(BufReader::new(std::io::empty())) as Box<dyn BufRead>
                        }
                    }
                }
            })
            .collect()
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    if serial {
        if let Err(e) = paste_serial(readers, &delimiters) {
            eprintln!("paste: {}", e);
            exit_code = 1;
        }
    } else {
        if let Err(e) = paste_parallel(readers, &delimiters) {
            eprintln!("paste: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

/// Strip the end-of-line terminator: `\n`, or the `\r\n` pair. A `\r` that is
/// not part of the terminator (e.g. a lone `\r` on the unterminated last
/// line) is kept.
fn strip_eol(line: &str) -> &str {
    let s = line.strip_suffix('\n').unwrap_or(line);
    s.strip_suffix('\r').unwrap_or(s)
}

/// Delimiter between output column `index` and the next one, cycling through
/// the `-d` character list. Without `-d` (empty list) there is no separator.
fn delimiter_for(delimiters: &[char], index: usize) -> String {
    if delimiters.is_empty() {
        return String::new();
    }
    delimiters[index % delimiters.len()].to_string()
}

fn paste_parallel(
    mut readers: Vec<Box<dyn BufRead>>,
    delimiters: &[char],
) -> Result<(), std::io::Error> {
    let mut lines: Vec<Option<String>> = readers.iter_mut().map(|_| None).collect();
    let mut eof_flags: Vec<bool> = readers.iter().map(|_| false).collect();
    let mut active_readers = readers.len();

    loop {
        for (i, reader) in readers.iter_mut().enumerate() {
            if !eof_flags[i] && lines[i].is_none() {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        eof_flags[i] = true;
                        active_readers -= 1;
                    }
                    Ok(_) => {
                        lines[i] = Some(strip_eol(&line).to_string());
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        if active_readers == 0 {
            break;
        }

        let mut output = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                output.push_str(&delimiter_for(delimiters, i - 1));
            }
            output.push_str(line.as_deref().unwrap_or(""));
        }
        println!("{}", output);

        for line in lines.iter_mut() {
            *line = None;
        }
    }

    Ok(())
}

fn paste_serial(
    mut readers: Vec<Box<dyn BufRead>>,
    delimiters: &[char],
) -> Result<(), std::io::Error> {
    for reader in readers.iter_mut() {
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();

        loop {
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    lines.push(strip_eol(&line).to_string());
                    line.clear();
                }
                Err(e) => return Err(e),
            }
        }

        if !lines.is_empty() {
            let mut output = String::new();
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    output.push_str(&delimiter_for(delimiters, i - 1));
                }
                output.push_str(line);
            }
            println!("{}", output);
        }
    }

    Ok(())
}
