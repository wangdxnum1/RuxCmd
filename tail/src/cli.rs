use clap::Parser;
use std::path::PathBuf;

/// Output the last part of files — a Windows port of the Linux tail command.
#[derive(Parser, Debug)]
#[command(name = "tail", version, about)]
pub struct Args {
    /// Print the last N lines instead of the last 10
    #[arg(short = 'n', long = "lines", default_value = "10")]
    pub lines: i64,

    /// Print the last N bytes
    #[arg(short = 'c', long = "bytes")]
    pub bytes: Option<i64>,

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