use clap::Parser;
use std::path::PathBuf;

/// Create named pipes — a Windows port of the Linux mkfifo command.
#[derive(Parser, Debug)]
#[command(name = "mkfifo", version, about, disable_version_flag = true)]
pub struct Args {
    /// Set file permission bits (not implemented on Windows)
    #[arg(short = 'm', long = "mode")]
    pub mode: Option<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// The named pipe(s) to create
    #[arg(value_name = "PIPE")]
    pub pipes: Vec<PathBuf>,
}
