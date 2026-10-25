use clap::Parser;
use std::path::PathBuf;

/// Remove empty directories — a Windows port of the Linux rmdir command.
#[derive(Parser, Debug)]
#[command(name = "rmdir", version, about)]
pub struct Args {
    /// Remove DIRECTORY and its ancestors
    #[arg(short = 'p', long = "parents")]
    pub parents: bool,

    /// Ignore each failure that is solely because a directory is non-empty
    #[arg(long = "ignore-fail-on-non-empty")]
    pub ignore_fail_on_non_empty: bool,

    /// Output a diagnostic for every directory processed
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The empty directory(ies) to remove
    #[arg(value_name = "DIRECTORY")]
    pub directories: Vec<PathBuf>,
}
