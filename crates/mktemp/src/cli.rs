use clap::Parser;
use std::path::PathBuf;

/// Create a temporary file or directory — a Windows port of the Linux mktemp command.
#[derive(Parser, Debug)]
#[command(name = "mktemp", version, about)]
pub struct Args {
    /// Create a directory, not a file
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Fail silently if an error occurs
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Interpret TEMPLATE relative to DIR
    #[arg(short = 'p', long = "tmpdir", value_name = "DIR")]
    pub tmpdir: Option<PathBuf>,

    /// Append SUFFIX to TEMPLATE; SUFFIX must not contain a slash
    #[arg(long = "suffix", value_name = "SUFFIX")]
    pub suffix: Option<String>,

    /// TEMPLATE — the run of trailing X's (3 or more) is replaced with
    /// random characters; defaults to tmp.XXXXXXXXXX in %TEMP%
    #[arg(value_name = "TEMPLATE")]
    pub templates: Vec<String>,
}
