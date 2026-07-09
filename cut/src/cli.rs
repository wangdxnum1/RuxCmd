use clap::Parser;
use std::path::PathBuf;

/// Remove sections from lines of files — a Windows port of the Linux cut command.
#[derive(Parser, Debug)]
#[command(name = "cut", version, about, disable_version_flag = true)]
pub struct Args {
    /// Select only these bytes
    #[arg(short = 'b', long = "bytes")]
    pub bytes: Option<String>,

    /// Select only these characters
    #[arg(short = 'c', long = "characters")]
    pub characters: Option<String>,

    /// Select only these fields
    #[arg(short = 'f', long = "fields")]
    pub fields: Option<String>,

    /// Use a specific delimiter
    #[arg(short = 'd', long = "delimiter")]
    pub delimiter: Option<char>,

    /// Only print lines containing the delimiter
    #[arg(short = 's', long = "only-delimited")]
    pub only_delimited: bool,

    /// Use a different output delimiter
    #[arg(short = 'o', long = "output-delimiter")]
    pub output_delimiter: Option<String>,

    /// The file(s) to cut
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}