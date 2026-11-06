use clap::Parser;
use std::path::PathBuf;

/// Estimate file space usage — a Windows port of the Linux du command.
#[derive(Parser, Debug)]
#[command(
    name = "du",
    version,
    about,
    disable_version_flag = true,
    disable_help_flag = true
)]
/// Help stays a long-only flag: -h keeps its GNU meaning in this command.
pub struct Args {
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    /// Show human-readable sizes (1.0K, 4.9K, 10K, 1.0M)
    #[arg(short = 'h', long = "human-readable")]
    pub human_readable: bool,

    /// Print sizes in bytes instead of 1K blocks (implies --apparent-size)
    #[arg(short = 'b', long = "bytes")]
    pub bytes: bool,

    /// Count apparent (logical) sizes instead of allocated disk space
    #[arg(long = "apparent-size")]
    pub apparent_size: bool,

    /// Write counts for all files, not just directories
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Show totals for all arguments
    #[arg(short = 's', long = "summarize")]
    pub summarize: bool,

    /// Print the total for a directory (or file, with --all) only if it is
    /// N or fewer levels below the command line argument; --max-depth=0 is
    /// the same as --summarize
    #[arg(short = 'd', long = "max-depth", value_name = "N")]
    pub max_depth: Option<String>,

    /// Skip directories on different file systems
    #[arg(short = 'x', long = "one-file-system")]
    pub one_file_system: bool,

    /// Exclude files that match PATTERN (repeatable)
    #[arg(long = "exclude", value_name = "PATTERN")]
    pub exclude: Vec<String>,

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
