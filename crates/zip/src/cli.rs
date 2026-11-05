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

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "ZIPFILE")]
    pub zipfile: PathBuf,

    #[arg(value_name = "FILES", num_args = 1.., required = true)]
    pub files: Vec<PathBuf>,
}
