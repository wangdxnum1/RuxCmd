use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "comm",
    about = "Compare two sorted files line by line",
    version = "0.1.0",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Show version")]
    pub version: bool,

    #[arg(short = '1', help = "Suppress lines unique to file 1")]
    pub hide_col1: bool,

    #[arg(short = '2', help = "Suppress lines unique to file 2")]
    pub hide_col2: bool,

    #[arg(short = '3', help = "Suppress lines that appear in both files")]
    pub hide_col3: bool,

    #[arg(help = "First sorted file to compare")]
    pub file1: Option<String>,

    #[arg(help = "Second sorted file to compare")]
    pub file2: Option<String>,
}
