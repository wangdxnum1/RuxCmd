mod cli;

use clap::Parser;
use std::io::{self, IsTerminal};

fn get_tty_name() -> Option<String> {
    std::env::var("CON").ok()
}

fn main() {
    let args = cli::Args::parse();

    if args.all {
        if !args.silent {
            let stdin_tty = if io::stdin().is_terminal() {
                get_tty_name().unwrap_or_else(|| "not a tty".to_string())
            } else {
                "not a tty".to_string()
            };
            let stdout_tty = if io::stdout().is_terminal() {
                get_tty_name().unwrap_or_else(|| "not a tty".to_string())
            } else {
                "not a tty".to_string()
            };
            let stderr_tty = if io::stderr().is_terminal() {
                get_tty_name().unwrap_or_else(|| "not a tty".to_string())
            } else {
                "not a tty".to_string()
            };
            println!("{}", stdin_tty);
            println!("{}", stdout_tty);
            println!("{}", stderr_tty);
        }
        std::process::exit(0);
    }

    if let Ok(tty) = std::env::var("CON") {
        if !args.silent {
            println!("{}", tty);
        }
        std::process::exit(0);
    } else {
        if !args.silent {
            println!("not a tty");
        }
        std::process::exit(1);
    }
}
