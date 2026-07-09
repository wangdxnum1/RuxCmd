use clap::Parser;
use std::path::PathBuf;

/// Print newline, word, and byte counts for files — a Windows port of the Linux wc command.
#[derive(Parser, Debug)]
#[command(name = "wc", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print the newline counts
    #[arg(short = 'l', long = "lines")]
    pub lines: bool,

    /// Print the word counts
    #[arg(short = 'w', long = "words")]
    pub words: bool,

    /// Print the byte counts
    #[arg(short = 'c', long = "bytes")]
    pub bytes: bool,

    /// Print the character counts
    #[arg(short = 'm', long = "chars")]
    pub chars: bool,

    /// Use the maximum width needed for all files
    #[arg(short = 'L', long = "max-line-length")]
    pub max_line_length: bool,

    /// The file(s) to count
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}