use clap::Parser;
use std::path::PathBuf;

/// Output the first part of files — a Windows port of the Linux head command.
#[derive(Parser, Debug)]
#[command(name = "head", version, about)]
pub struct Args {
    /// Use NUL instead of newline as the record delimiter
    #[arg(short = 'z', long = "zero-terminated")]
    pub zero_terminated: bool,
    /// Print the first N lines instead of the first 10; a negative -N
    /// prints all but the last N lines
    #[arg(
        short = 'n',
        long = "lines",
        value_name = "N",
        allow_negative_numbers = true
    )]
    pub lines: Option<String>,

    /// Print the first N bytes; a negative -N prints all but the last N bytes
    #[arg(
        short = 'c',
        long = "bytes",
        value_name = "N",
        allow_negative_numbers = true
    )]
    pub bytes: Option<String>,

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
