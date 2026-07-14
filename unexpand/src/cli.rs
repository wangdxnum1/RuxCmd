use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "unexpand", version, about = "Convert spaces in FILE(s) to tabs", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "tabs", default_value_t = 8usize)]
    pub tabs: usize,

    #[arg(short = 'a', long = "all", action = clap::ArgAction::SetTrue)]
    pub all: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
