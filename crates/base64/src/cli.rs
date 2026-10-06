use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "base64",
    version,
    about = "Base64 encode/decode FILE(s) to stdout",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'd', long = "decode", action = clap::ArgAction::SetTrue)]
    pub decode: bool,

    #[arg(short = 'w', long = "wrap", default_value_t = 76)]
    pub wrap: usize,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
