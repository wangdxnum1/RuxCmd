use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "csplit",
    version,
    about = "Split FILE by PATTERN to xxNN output files",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'f', long = "prefix", default_value = "xx")]
    pub prefix: String,

    #[arg(short = 'b', long = "suffix-format", default_value = "%02d")]
    pub suffix_format: String,

    #[arg(short = 'n', long = "digits", default_value_t = 2usize)]
    pub digits: usize,

    #[arg(short = 'k', long = "keep-files", action = clap::ArgAction::SetTrue)]
    pub keep: bool,

    #[arg(short = 'z', long = "elide-empty-files", action = clap::ArgAction::SetTrue)]
    pub elide_empty: bool,

    #[arg(short = 's', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    #[arg(value_name = "PATTERNS", last = true)]
    pub patterns: Vec<String>,
}
