use clap::Parser;
use std::path::PathBuf;

/// Create links — a Windows port of the Linux ln command.
#[derive(Parser, Debug)]
#[command(name = "ln", version, about, disable_version_flag = true)]
pub struct Args {
    /// Create symbolic links instead of hard links
    #[arg(short = 's', long = "symbolic")]
    pub symbolic: bool,

    /// Remove existing destination files
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Source file/directory
    #[arg(value_name = "TARGET")]
    pub target: PathBuf,

    /// Link name or directory
    #[arg(value_name = "LINK_NAME")]
    pub link_name: PathBuf,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}