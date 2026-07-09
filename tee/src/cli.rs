use clap::Parser;
use std::path::PathBuf;

/// Read from standard input and write to standard output and files — a Windows port of the Linux tee command.
#[derive(Parser, Debug)]
#[command(name = "tee", version, about, disable_version_flag = true)]
pub struct Args {
    /// Append to the given files rather than overwriting
    #[arg(short = 'a', long = "append")]
    pub append: bool,

    /// Ignore interrupt signals
    #[arg(short = 'i', long = "ignore-interrupts")]
    pub ignore_interrupts: bool,

    /// The file(s) to write to
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}