use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "zip", version, about, disable_version_flag = true)]
pub struct Args {
    /// Recurse into directories
    #[arg(short = 'r', long = "recurse")]
    pub recurse: bool,

    /// Quiet mode: suppress per-entry messages
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Replace existing archive entries only when the source is newer
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Store just the name of a saved file (junk the path), not directories
    #[arg(short = 'j', long = "junk-paths")]
    pub junk_paths: bool,

    /// Store files without deflate compression
    #[arg(short = '0', long = "store")]
    pub store: bool,

    /// Deflate at the slowest, smallest level (-9); default is level 6
    #[arg(short = '9')]
    pub best: bool,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "ZIPFILE")]
    pub zipfile: PathBuf,

    #[arg(value_name = "FILES", num_args = 1.., required = true)]
    pub files: Vec<PathBuf>,
}
