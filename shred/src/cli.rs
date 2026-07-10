use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "shred", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'f', long = "force", help = "Force permissions to allow writing")]
    pub force: bool,

    #[arg(short = 'n', long = "iterations", default_value = "3", help = "Number of overwrite iterations")]
    pub iterations: usize,

    #[arg(short = 'u', long = "remove", help = "Remove file after shredding")]
    pub remove: bool,

    #[arg(short = 'v', long = "verbose", help = "Verbose output")]
    pub verbose: bool,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}