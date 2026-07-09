use clap::Parser;
use std::path::PathBuf;

/// Search for patterns in files — a Windows port of the Linux grep command.
#[derive(Parser, Debug)]
#[command(name = "grep", version, about)]
pub struct Args {
    /// Print only the count of matching lines
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Print line numbers with output lines
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Suppress normal output; exit status indicates match
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Invert match: select non-matching lines
    #[arg(short = 'v', long = "invert-match")]
    pub invert_match: bool,

    /// Recursively search subdirectories
    #[arg(short = 'r', short_alias = 'R', long = "recursive")]
    pub recursive: bool,

    /// Ignore case distinctions
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Match the pattern only at the beginning of lines
    #[arg(short = 'x', long = "line-regexp")]
    pub line_regexp: bool,

    /// Show only the part of the line matching the pattern
    #[arg(short = 'o', long = "only-matching")]
    pub only_matching: bool,

    /// The pattern to search for
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// The file(s) to search
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}