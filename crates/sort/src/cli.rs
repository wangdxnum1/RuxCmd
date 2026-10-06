use clap::Parser;
use std::path::PathBuf;

/// Sort lines of text — a Windows port of the Linux sort command.
#[derive(Parser, Debug)]
#[command(name = "sort", version, about, disable_version_flag = true)]
pub struct Args {
    /// Sort in reverse order
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// Sort by numeric value
    #[arg(short = 'n', long = "numeric-sort")]
    pub numeric: bool,

    /// Ignore case distinctions
    #[arg(short = 'f', long = "ignore-case")]
    pub ignore_case: bool,

    /// Sort by human-readable numbers (e.g., 1K, 2M)
    #[arg(short = 'h', long = "human-numeric-sort")]
    pub human_numeric: bool,

    /// Remove duplicate lines
    #[arg(short = 'u', long = "unique")]
    pub unique: bool,

    /// Check if input is sorted
    #[arg(short = 'c', long = "check")]
    pub check: bool,

    /// Sort by column (1-indexed)
    #[arg(short = 'k', long = "key")]
    pub key: Option<String>,

    /// Use a specific delimiter
    #[arg(short = 't', long = "field-separator")]
    pub delimiter: Option<char>,

    /// The file(s) to sort
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
