use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "expand", version, about = "Convert tabs in FILE(s) to spaces", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "tabs", default_value = "8")]
    pub tabs: String,

    #[arg(short = 'i', long = "initial", action = clap::ArgAction::SetTrue)]
    pub initial: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
