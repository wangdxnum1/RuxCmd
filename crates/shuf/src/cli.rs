use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "shuf",
    version,
    about = "Shuffle input lines randomly",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'i', long = "input-range", value_name = "LO-HI")]
    pub input_range: Option<String>,

    #[arg(short = 'n', long = "head-count", default_value_t = usize::MAX)]
    pub head_count: usize,

    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[arg(long = "random-source", value_name = "SEEDSTR")]
    pub random_source: Option<String>,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
