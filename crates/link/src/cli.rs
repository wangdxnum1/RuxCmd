use clap::Parser;
use std::path::PathBuf;

/// Create a hard link — a Windows port of the Linux link command.
#[derive(Parser, Debug)]
#[command(name = "link", version, about)]
pub struct Args {
    /// FILE1 FILE2 — create a link to FILE1 with the name FILE2
    #[arg(value_name = "FILE")]
    pub operands: Vec<PathBuf>,
}
