use clap::Parser;
use std::path::PathBuf;

/// Report file system disk space usage — a Windows port of the Linux df command.
#[derive(Parser, Debug)]
#[command(
    name = "df",
    version,
    about,
    disable_version_flag = true,
    disable_help_flag = true
)]
/// Help stays a long-only flag: -h keeps its GNU meaning in this command.
pub struct Args {
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    /// Show human-readable sizes (KB, MB, GB)
    #[arg(short = 'h', long = "human-readable")]
    pub human_readable: bool,

    /// Include the file-system type column
    #[arg(short = 'T', long = "print-type")]
    pub print_type: bool,

    /// File system paths to report on
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
