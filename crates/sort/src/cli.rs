use clap::Parser;
use std::path::PathBuf;

/// Sort lines of text — a Windows port of the Linux sort command.
#[derive(Parser, Debug)]
#[command(
    name = "sort",
    version,
    about,
    disable_version_flag = true,
    disable_help_flag = true
)]
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

    /// Sort by human-readable numbers (e.g., 1K, 2M), 1024-based
    #[arg(short = 'h', long = "human-numeric-sort")]
    pub human_numeric: bool,

    /// Remove lines whose sort keys compare equal (applied after sorting)
    #[arg(short = 'u', long = "unique")]
    pub unique: bool,

    /// Check that input is sorted (with -r: reverse-sorted)
    #[arg(short = 'c', long = "check")]
    pub check: bool,

    /// Sort using key fields F1 through F2 (1-indexed, inclusive); without
    /// `,F2` the key runs from field F1 to end of line. Field-character
    /// offsets and per-key modifiers are not supported.
    #[arg(short = 'k', long = "key", value_name = "F1[,F2]")]
    pub key: Option<String>,

    /// Use SEP (a single character) as the field separator; without SEP,
    /// fields are split on runs of blanks
    #[arg(short = 't', long = "field-separator", value_name = "SEP")]
    pub delimiter: Option<String>,

    /// The file(s) to sort
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),
}
