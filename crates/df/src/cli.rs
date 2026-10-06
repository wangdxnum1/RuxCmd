use clap::Parser;
use std::path::PathBuf;

/// Report file system disk space usage — a Windows port of the Linux df command.
#[derive(Parser, Debug)]
#[command(name = "df", version, about, disable_version_flag = true)]
pub struct Args {
    /// Show human-readable sizes (KB, MB, GB)
    #[arg(short = 'h', long = "human-readable")]
    pub human_readable: bool,

    /// File system paths to report on
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
