use clap::Parser;
use std::path::PathBuf;

/// Move or rename files and directories — a Windows port of the Linux mv command.
#[derive(Parser, Debug)]
#[command(name = "mv", version, about)]
pub struct Args {
    /// Overwrite existing files without prompting
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Prompt before overwrite
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Do not overwrite an existing file
    #[arg(short = 'n', long = "no-clobber")]
    pub no_clobber: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The source file(s) to move
    #[arg(value_name = "SOURCE")]
    pub sources: Vec<PathBuf>,

    /// The destination file or directory
    #[arg(value_name = "DESTINATION")]
    pub destination: PathBuf,
}