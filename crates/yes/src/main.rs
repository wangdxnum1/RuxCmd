mod cli;

use clap::Parser;
use std::io::{self, BufWriter, Write};

fn main() {
    let args = cli::Args::parse();

    let line = if args.strings.is_empty() {
        "y".to_string()
    } else {
        args.strings.join(" ")
    };

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    // GNU yes dies silently on EPIPE; a closed pipe is a normal exit here.
    loop {
        for _ in 0..256 {
            if let Err(e) = writeln!(out, "{}", line) {
                handle_write_error(e);
            }
        }
        if let Err(e) = out.flush() {
            handle_write_error(e);
        }
    }
}

fn handle_write_error(e: io::Error) -> ! {
    if e.kind() == io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }
    eprintln!("yes: {}", e);
    std::process::exit(1);
}
