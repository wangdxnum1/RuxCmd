use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bc", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Accepted for GNU compatibility; the math library is not built in.
    #[arg(short = 'l')]
    pub mathlib: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
