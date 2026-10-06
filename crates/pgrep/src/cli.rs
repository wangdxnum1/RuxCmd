use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pgrep", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'f', long = "full", help = "Match full command line")]
    pub full: bool,

    #[arg(short = 'l', long = "list-name", help = "List process names")]
    pub list_name: bool,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "PATTERN")]
    pub pattern: String,
}
