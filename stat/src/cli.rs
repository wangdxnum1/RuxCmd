use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "stat", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'c', long = "format")]
    pub format: Option<String>,

    #[arg(short = 'f', long = "file-system")]
    pub file_system: bool,

    #[arg(short = 't', long = "terse")]
    pub terse: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
