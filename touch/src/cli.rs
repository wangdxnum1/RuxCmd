use clap::Parser;
use std::path::PathBuf;

/// Create empty files or update file timestamps — a Windows port of the Linux touch command.
#[derive(Parser, Debug)]
#[command(name = "touch", version, about)]
pub struct Args {
    /// Do not create any files
    #[arg(short = 'c', long = "no-create")]
    pub no_create: bool,

    /// The file(s) to touch
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}