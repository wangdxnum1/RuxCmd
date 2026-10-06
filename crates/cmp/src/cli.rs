use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "cmp",
    about = "Compare two files byte by byte",
    version = "0.1.0",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Show version")]
    pub version: bool,

    #[arg(
        short = 'l',
        long,
        help = "Print the byte number and differing byte values"
    )]
    pub verbose: bool,

    #[arg(short = 's', long, help = "Suppress all output")]
    pub silent: bool,

    #[arg(help = "First file to compare")]
    pub file1: Option<String>,

    #[arg(help = "Second file to compare")]
    pub file2: Option<String>,
}
