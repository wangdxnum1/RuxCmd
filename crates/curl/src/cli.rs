use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "curl",
    version,
    about = "Transfer data from or to a server",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'X', long = "request", default_value = "GET")]
    pub method: String,

    #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    #[arg(short = 'd', long = "data")]
    pub data: Option<String>,

    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    #[arg(short = 's', long = "silent", action = clap::ArgAction::SetTrue)]
    pub silent: bool,

    #[arg(value_name = "URL")]
    pub url: Option<String>,
}
