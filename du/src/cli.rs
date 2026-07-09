use clap::Parser;
use std::path::PathBuf;

/// Estimate file space usage — a Windows port of the Linux du command.
#[derive(Parser, Debug)]
#[command(name = "du", version, about, disable_version_flag = true)]
pub struct Args {
    /// Show human-readable sizes (KB, MB, GB)
    #[arg(short = 'h', long = "human-readable")]
    pub human_readable: bool,

    /// Show totals for all arguments
    #[arg(short = 's', long = "summarize")]
    pub summarize: bool,

    /// Show grand total
    #[arg(short = 'c', long = "total")]
    pub total: bool,

    /// Directory or file paths to calculate
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}