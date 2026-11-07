use clap::Parser;
use std::path::PathBuf;

/// Remove sections from lines of files — a Windows port of the Linux cut command.
#[derive(Parser, Debug)]
#[command(name = "cut", version, about, disable_version_flag = true)]
pub struct Args {
    /// Use NUL instead of newline to separate input and output records
    #[arg(short = 'z', long = "zero-terminated")]
    pub zero_terminated: bool,
    /// Select only these bytes
    #[arg(short = 'b', long = "bytes")]
    pub bytes: Option<String>,

    /// With -b: do not split multibyte characters — a byte selection point
    /// that falls inside a character selects the whole character (input that
    /// is not valid UTF-8 falls back to raw byte selection)
    #[arg(short = 'n')]
    pub no_split: bool,

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

    /// Complement the set of selected bytes, characters or fields
    #[arg(long = "complement")]
    pub complement: bool,

    /// The file(s) to cut
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
