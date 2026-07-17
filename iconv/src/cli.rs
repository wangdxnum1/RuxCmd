use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "iconv", version, about = "Convert text from one encoding to another", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'f', long = "from-code")]
    pub from_code: Option<String>,

    #[arg(short = 't', long = "to-code")]
    pub to_code: Option<String>,

    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    #[arg(short = 'l', long = "list", action = clap::ArgAction::SetTrue)]
    pub list: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}