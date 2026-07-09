use clap::Parser;
use std::path::PathBuf;

/// Create directories — a Windows port of the Linux mkdir command.
#[derive(Parser, Debug)]
#[command(name = "mkdir", version, about)]
pub struct Args {
    /// Create parent directories as needed
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,

    /// Set file permission bits (not implemented on Windows)
    #[arg(short = 'm', long = "mode")]
    pub mode: Option<String>,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The directory(ies) to create
    #[arg(value_name = "DIRECTORY")]
    pub directories: Vec<PathBuf>,
}