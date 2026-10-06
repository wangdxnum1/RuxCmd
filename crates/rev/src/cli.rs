use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rev", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
