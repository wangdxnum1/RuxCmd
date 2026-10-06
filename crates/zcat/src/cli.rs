use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Decompress gzip files to stdout",
    long_about = None,
    arg_required_else_help = false,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Print version information")]
    pub version: bool,

    #[arg(help = "Input file(s) to decompress")]
    pub files: Vec<String>,
}
