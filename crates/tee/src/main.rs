mod cli;

use clap::Parser;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();

    let append = args.append;
    let ignore_interrupts = args.ignore_interrupts;

    if ignore_interrupts {
        #[cfg(unix)]
        {
            use std::os::unix::process::Signal;
            use std::process;
            unsafe {
                libc::signal(libc::SIGINT, libc::SIG_IGN);
            }
        }
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    let mut files: Vec<std::fs::File> = Vec::new();

    for path in &args.files {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(path)
        {
            Ok(file) => files.push(file),
            Err(e) => {
                eprintln!("tee: {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    let mut stdout = io::stdout();
    let mut exit_code = 0;

    for line in reader.lines() {
        match line {
            Ok(l) => {
                writeln!(stdout, "{}", l).unwrap_or_else(|e| {
                    eprintln!("tee: error writing to stdout: {}", e);
                    exit_code = 1;
                });

                for file in &mut files {
                    if let Err(e) = writeln!(file, "{}", l) {
                        eprintln!("tee: error writing to file: {}", e);
                        exit_code = 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("tee: error reading from stdin: {}", e);
                exit_code = 1;
                break;
            }
        }
    }

    for file in &mut files {
        if let Err(e) = file.flush() {
            eprintln!("tee: error flushing file: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}
