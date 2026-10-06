use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None, disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Display version information")]
    pub version: bool,
}
