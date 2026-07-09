use clap::Parser;
use std::path::PathBuf;

/// Search for files in a directory hierarchy — a Windows port of the Linux find command.
#[derive(Parser, Debug)]
#[command(name = "find", version, about)]
pub struct Args {
    /// Search paths (default: current directory)
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Match files by name pattern
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,

    /// Match files by regex pattern
    #[arg(long = "regex")]
    pub regex: Option<String>,

    /// Search only for directories
    #[arg(long = "type", value_name = "TYPE")]
    pub file_type: Option<char>,

    /// Minimum depth for search
    #[arg(long = "mindepth")]
    pub mindepth: Option<usize>,

    /// Maximum depth for search
    #[arg(long = "maxdepth")]
    pub maxdepth: Option<usize>,

    /// Print only filenames (not full paths)
    #[arg(short = 'f', long = "basename")]
    pub basename: bool,

    /// Search case-insensitive
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,
}