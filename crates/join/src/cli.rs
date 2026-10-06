use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Join lines of two files on a common field — a Windows port of the Linux join command.
#[derive(Parser, Debug)]
#[command(name = "join", version = "0.1.0", about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = ArgAction::Version, help = "Print version information")]
    pub version: (),

    /// Join on the FIELDth column of file1
    #[arg(short = '1', long, value_name = "FIELD", default_value = "1")]
    pub field1: usize,

    /// Join on the FIELDth column of file2
    #[arg(short = '2', long, value_name = "FIELD", default_value = "1")]
    pub field2: usize,

    /// Use CHAR as input and output field separator
    #[arg(short = 't', long = "separator", value_name = "CHAR")]
    pub delimiter: Option<char>,

    /// First file
    #[arg(value_name = "FILE1")]
    pub file1: PathBuf,

    /// Second file
    #[arg(value_name = "FILE2")]
    pub file2: PathBuf,
}
