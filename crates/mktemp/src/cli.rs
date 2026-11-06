use clap::Parser;
use std::path::PathBuf;

/// Create a temporary file or directory — a Windows port of the Linux mktemp command.
#[derive(Parser, Debug)]
#[command(name = "mktemp", version, about)]
pub struct Args {
    /// Create a directory, not a file
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Do not create anything; just print a name (unsafe)
    #[arg(short = 'u', long = "dry-run")]
    pub dry_run: bool,

    /// Fail silently if an error occurs
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Interpret TEMPLATE relative to DIR; without a value, use $TMPDIR
    /// (or %TEMP%). Like GNU mktemp, give the value as --tmpdir=DIR;
    /// a space-separated value is consumed greedily, so put the TEMPLATE
    /// before a bare --tmpdir.
    #[arg(short = 'p', long = "tmpdir", value_name = "DIR", num_args = 0..=1)]
    pub tmpdir: Option<Option<PathBuf>>,

    /// Interpret a filename template relative to the Windows temporary directory
    #[arg(short = 't')]
    pub use_temp_dir: bool,

    /// Append SUFFIX to TEMPLATE; SUFFIX must not contain a slash
    #[arg(long = "suffix", value_name = "SUFFIX")]
    pub suffix: Option<String>,

    /// TEMPLATE — the run of trailing X's (3 or more) is replaced with
    /// random characters; defaults to tmp.XXXXXXXXXX in %TEMP%
    #[arg(value_name = "TEMPLATE")]
    pub templates: Vec<String>,
}
