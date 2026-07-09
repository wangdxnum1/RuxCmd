use clap::Parser;
use std::path::PathBuf;

/// Output the first part of files — a Windows port of the Linux head command.
#[derive(Parser, Debug)]
#[command(name = "head", version, about)]
pub struct Args {
    /// Print the first N lines instead of the first 10
    #[arg(short = 'n', long = "lines", default_value = "10")]
    pub lines: i64,

    /// Print the first N bytes
    #[arg(short = 'c', long = "bytes")]
    pub bytes: Option<i64>,

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