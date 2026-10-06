use clap::Parser;
use std::path::PathBuf;

/// Change file permissions — a Windows port of the Linux chmod command.
#[derive(Parser, Debug)]
#[command(name = "chmod", version, about, disable_version_flag = true)]
pub struct Args {
    /// Change files and directories recursively
    #[arg(short = 'R', long = "recursive")]
    pub recursive: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The permission mode (e.g., 755, 644)
    #[arg(value_name = "MODE")]
    pub mode: String,

    /// The file(s) to change permissions for
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
