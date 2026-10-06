use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Merge lines of files — a Windows port of the Linux paste command.
#[derive(Parser, Debug)]
#[command(name = "paste", version = "0.1.0", about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = ArgAction::Version, help = "Print version information")]
    pub version: (),

    /// Use STRING as separator instead of TAB
    #[arg(short = 'd', long = "delimiters", value_name = "STRING")]
    pub delimiters: Option<String>,

    /// Serial mode: paste one file at a time instead of in parallel
    #[arg(short = 's', long = "serial")]
    pub serial: bool,

    /// The file(s) to merge
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
