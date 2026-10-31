use clap::Parser;
use std::io::Read;

/// Write messages to the system log — a Windows port of Linux logger.
#[derive(Parser, Debug)]
#[command(name = "logger", version, about)]
pub struct Args {
    /// Print the message to stderr as well
    #[arg(short = 's')]
    pub stderr: bool,

    /// Tag each line with the given identifier
    #[arg(short = 't', long = "tag", value_name = "TAG")]
    pub tag: Option<String>,

    /// Messages to log (default: read from stdin)
    #[arg(value_name = "MESSAGE")]
    pub messages: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let tag = args
        .tag
        .unwrap_or_else(|| std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string()));

    let text = if args.messages.is_empty() {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        args.messages.join(" ")
    };

    for line in text.lines() {
        let entry = format!("{}: {}", tag, line);
        if args.stderr {
            eprintln!("{}", entry);
        }
    }
}
