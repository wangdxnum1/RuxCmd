use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fmt", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'w', long = "width", default_value = "75")]
    pub width: usize,

    #[arg(short = 'c', long = "crown-margin")]
    pub crown_margin: Option<usize>,

    #[arg(short = 's', long = "split-only", action = clap::ArgAction::SetTrue)]
    pub split_only: bool,

    #[arg(default_value = "-")]
    pub files: Vec<String>,
}