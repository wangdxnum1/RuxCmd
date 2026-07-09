use clap::Parser;
use std::path::PathBuf;

/// Open files, directories, and URLs with their associated applications — a Windows port of the Mac open command.
#[derive(Parser, Debug)]
#[command(name = "open", version, about)]
pub struct Args {
    /// Open the file(s) with the specified application
    #[arg(short = 'a', long = "application")]
    pub application: Option<String>,

    /// Open with Notepad
    #[arg(short = 'e', long = "edit")]
    pub edit: bool,

    /// Wait until the opened application exits
    #[arg(short = 'W', long = "wait")]
    pub wait: bool,

    /// Reveal the file in File Explorer instead of opening it
    #[arg(short = 'R', long = "reveal")]
    pub reveal: bool,

    /// The file(s), directory, or URL to open
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}