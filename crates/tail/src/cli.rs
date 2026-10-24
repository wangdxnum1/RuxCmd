use clap::Parser;
use std::path::PathBuf;

/// Output the last part of files — a Windows port of the Linux tail command.
#[derive(Parser, Debug)]
#[command(name = "tail", version, about)]
pub struct Args {
    /// Print the last N lines instead of the last 10; `+N` prints from
    /// line N onward
    #[arg(short = 'n', long = "lines", default_value = "10", value_name = "N")]
    pub lines: String,

    /// Print the last N bytes; `+N` prints from byte N onward
    #[arg(short = 'c', long = "bytes", value_name = "N")]
    pub bytes: Option<String>,

    /// Output appended data as the file grows
    #[arg(short = 'f', long = "follow")]
    pub follow: bool,

    /// Never print headers giving file names
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Always print headers giving file names
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The file(s) to read
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

/// A `-n`/`-c` argument: print the last N units, or (`+N`) from unit N on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Count {
    /// The last N lines/bytes (N = 0 prints nothing).
    Last(usize),
    /// Everything from line/byte N onward (`+N`, 1-indexed; N <= 1 is the
    /// whole input).
    From(usize),
}

impl Count {
    /// Parses a raw `-n`/`-c` value: `+N` selects "from N onward", a plain
    /// or negative number selects "last N" (negative clamps to 0; GNU's
    /// "all but the last N" is not implemented). Unparsable values exit 1
    /// with GNU's diagnostic.
    pub fn parse(raw: &str, option: char) -> Count {
        let (digits, from_start) = match raw.strip_prefix('+') {
            Some(rest) => (rest, true),
            None => (raw, false),
        };
        let n = match digits.parse::<i64>() {
            Ok(n) => n.max(0) as usize,
            Err(_) => {
                let unit = if option == 'n' { "lines" } else { "bytes" };
                eprintln!("tail: invalid number of {}: '{}'", unit, raw);
                std::process::exit(1);
            }
        };
        if from_start {
            Count::From(n)
        } else {
            Count::Last(n)
        }
    }
}
