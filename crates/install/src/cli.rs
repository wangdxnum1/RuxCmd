use clap::Parser;
use std::path::PathBuf;

/// Copy files and set attributes — a Windows port of the Linux install command.
#[derive(Parser, Debug)]
#[command(name = "install", version, about)]
pub struct Args {
    /// Create directories instead of copying files
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Create missing parent directories of the destination
    #[arg(short = 'D')]
    pub create_parents: bool,

    /// Set file mode (octal, mapped onto the read-only attribute)
    #[arg(short = 'm', long = "mode")]
    pub mode: Option<String>,

    /// Print the name of each created file or directory
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Source files, followed by the destination
    #[arg(value_name = "SOURCE... DEST")]
    pub operands: Vec<PathBuf>,
}
