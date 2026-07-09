use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "diff",
    about = "Compare two files line by line",
    version = "0.1.0",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Show version")]
    pub version: bool,

    #[arg(short = 'u', long, help = "Output unified diff format")]
    pub unified: bool,

    #[arg(short = 'i', long, help = "Ignore case differences")]
    pub ignore_case: bool,

    #[arg(short = 'b', long, help = "Ignore whitespace differences")]
    pub ignore_whitespace: bool,

    #[arg(short = 'B', long, help = "Ignore blank lines")]
    pub ignore_blank_lines: bool,

    #[arg(help = "First file to compare")]
    pub file1: Option<String>,

    #[arg(help = "Second file to compare")]
    pub file2: Option<String>,
}