use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "column",
    version,
    about = "Columnate lists",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "table", action = clap::ArgAction::SetTrue)]
    pub table: bool,

    // None = GNU default: split on runs of whitespace (space/tab) with runs
    // collapsed; Some(seps) = split on each separator character, no collapsing.
    #[arg(short = 's', long = "separator", value_name = "SEP")]
    pub separator: Option<String>,

    #[arg(short = 'o', long = "output-separator", default_value = "  ")]
    pub output_separator: String,

    #[arg(short = 'R', long = "table-right", value_name = "COLS")]
    pub table_right: Option<String>,

    #[arg(short = 'H', long = "table-header-repeat", value_name = "LINES")]
    pub header: Option<usize>,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
