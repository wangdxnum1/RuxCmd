use clap::Parser;
use std::path::PathBuf;

/// Search for patterns in files — a Windows port of the Linux grep command.
#[derive(Parser, Debug)]
#[command(name = "grep", version, about, disable_help_flag = true)]
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

    /// Use extended regular expressions (the regex engine is already ERE-like,
    /// so this flag changes nothing and is kept for GNU compatibility)
    #[arg(short = 'E', long = "extended-regexp")]
    pub extended_regexp: bool,

    /// Match using fixed strings, not regular expressions
    #[arg(short = 'F', long = "fixed-strings")]
    pub fixed_strings: bool,

    /// Match only whole words
    #[arg(short = 'w', long = "word-regexp")]
    pub word_regexp: bool,

    /// Print only the names of files with matching lines
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,

    /// Suppress the file name prefix on output lines
    #[arg(short = 'h', long = "no-filename")]
    pub no_filename: bool,

    /// Always print the file name prefix on output lines
    #[arg(short = 'H', long = "with-filename")]
    pub with_filename: bool,

    /// Print NUM lines of trailing context
    #[arg(short = 'A', long = "after-context", value_name = "NUM")]
    pub after_context: Option<usize>,

    /// Print NUM lines of leading context
    #[arg(short = 'B', long = "before-context", value_name = "NUM")]
    pub before_context: Option<usize>,

    /// Print NUM lines of leading and trailing context
    #[arg(short = 'C', long = "context", value_name = "NUM")]
    pub context: Option<usize>,

    /// Show help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    /// The pattern to search for
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// The file(s) to search
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
