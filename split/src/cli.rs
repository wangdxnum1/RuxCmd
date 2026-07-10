use clap::Parser;
use std::path::PathBuf;

/// Split a file into pieces — a Windows port of the Linux split command.
#[derive(Parser, Debug)]
#[command(name = "split", about, long_about = None, version = "0.1.0", disable_version_flag = true)]
pub struct Args {
    /// Split file into chunks of SIZE bytes
    #[arg(short = 'b', long = "bytes", value_name = "SIZE")]
    pub bytes: Option<String>,

    /// Split file into chunks of N lines
    #[arg(short = 'l', long = "lines", value_name = "N")]
    pub lines: Option<usize>,

    /// Split file into N pieces
    #[arg(short = 'n', long = "number", value_name = "N")]
    pub number: Option<usize>,

    /// Use numeric suffixes instead of alphabetic
    #[arg(short = 'd', long = "numeric-suffixes")]
    pub numeric_suffix: bool,

    /// Use suffixes of length N
    #[arg(short = 'a', long = "suffix-length", default_value = "2")]
    pub suffix_length: usize,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue, global = true)]
    pub version: bool,

    /// Output file prefix
    #[arg(short = 'o', long = "output", default_value = "x")]
    pub output: String,

    /// The file to split
    #[arg(value_name = "FILE", required_unless_present = "version")]
    pub file: Option<PathBuf>,
}