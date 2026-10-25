use clap::Parser;
use std::path::PathBuf;

/// Remove files and symbolic links — a Windows port of the Linux unlink command.
#[derive(Parser, Debug)]
#[command(name = "unlink", version, about)]
pub struct Args {
    /// FILE — the file or symbolic link to remove
    #[arg(value_name = "FILE")]
    pub operands: Vec<PathBuf>,
}
