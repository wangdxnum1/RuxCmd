use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hexdump", version, about = "Display file contents in hexadecimal", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'C', long = "canonical", action = clap::ArgAction::SetTrue)]
    pub canonical: bool,

    #[arg(short = 'd', long = "decimal", action = clap::ArgAction::SetTrue)]
    pub decimal: bool,

    #[arg(short = 'o', long = "octal", action = clap::ArgAction::SetTrue)]
    pub octal: bool,

    #[arg(short = 'x', long = "hexadecimal", action = clap::ArgAction::SetTrue)]
    pub hexadecimal: bool,

    #[arg(short = 'c', long = "ascii", action = clap::ArgAction::SetTrue)]
    pub ascii: bool,

    #[arg(short = 's', long = "skip")]
    pub skip: Option<usize>,

    #[arg(short = 'n', long = "length")]
    pub length: Option<usize>,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}