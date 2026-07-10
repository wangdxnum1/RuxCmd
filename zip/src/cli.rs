use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "zip", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'r', long = "recurse", help = "Recurse into directories")]
    pub recurse: bool,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "ZIPFILE")]
    pub zipfile: PathBuf,

    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,
}