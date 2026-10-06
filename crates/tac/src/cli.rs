use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tac",
    version,
    about = "Write FILE(s) to stdout, last line first",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'b', long = "before", action = clap::ArgAction::SetTrue)]
    pub before: bool,

    #[arg(short = 's', long = "separator", default_value_t = String::from("\n"))]
    pub separator: String,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
