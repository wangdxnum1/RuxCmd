use clap::Parser;
use std::path::PathBuf;

/// Copy files and directories — a Windows port of the Linux cp command.
#[derive(Parser, Debug)]
#[command(name = "cp", version, about)]
pub struct Args {
    /// Copy directories recursively
    #[arg(short = 'r', short_alias = 'R', long = "recursive")]
    pub recursive: bool,

    /// Overwrite existing files without prompting
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Prompt before overwrite
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Preserve file attributes
    #[arg(short = 'p', long = "preserve")]
    pub preserve: bool,

    /// Create symbolic links instead of copying
    #[arg(short = 's', long = "symbolic-link")]
    pub symbolic_link: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The source file(s) to copy
    #[arg(value_name = "SOURCE", required = true, num_args = 1..)]
    pub sources: Vec<PathBuf>,

    /// The destination file or directory
    #[arg(value_name = "DESTINATION")]
    pub destination: PathBuf,
}
