use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long)]
    pub version: bool,

    #[arg(short, long)]
    pub brief: bool,

    #[arg(short = 'i', long)]
    pub mime: bool,

    pub files: Vec<String>,
}
