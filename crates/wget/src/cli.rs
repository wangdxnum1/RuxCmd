use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "wget",
    version,
    about = "Non-interactive network downloader",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'O', long = "output-document")]
    pub output: Option<PathBuf>,

    #[arg(short = 'P', long = "directory-prefix")]
    pub directory: Option<PathBuf>,

    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    #[arg(value_name = "URL")]
    pub url: Option<String>,
}
