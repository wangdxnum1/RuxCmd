use clap::Parser;
use std::path::PathBuf;

/// Stream editor for filtering and transforming text — a Windows port of the Linux sed command.
#[derive(Parser, Debug)]
#[command(name = "sed", version, about, disable_version_flag = true)]
pub struct Args {
    /// Add the script to the commands to be executed
    #[arg(short = 'e', long = "expression", value_name = "SCRIPT")]
    pub expressions: Vec<String>,

    /// Edit files in place (backup if extension supplied) empty string means no backup)
    #[arg(short = 'i', long = "in-place", value_name = "SUFFIX")]
    pub in_place: Option<String>,

    /// Consider files as separate rather than as a single continuous stream
    #[arg(short = 's', long = "separate")]
    pub separate: bool,

    /// The file(s) to edit
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
