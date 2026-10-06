use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fold", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'w', long = "width", default_value = "80")]
    pub width: usize,

    #[arg(short = 'b', long = "bytes", action = clap::ArgAction::SetTrue)]
    pub bytes: bool,

    #[arg(short = 's', long = "spaces", action = clap::ArgAction::SetTrue)]
    pub spaces: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
