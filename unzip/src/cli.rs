use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "unzip", version = "0.1.0", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'l', long = "list", help = "List files in archive")]
    pub list: bool,

    #[arg(short = 'd', long = "directory", help = "Extract files into specified directory")]
    pub directory: Option<String>,

    #[arg(short = 'o', long = "overwrite", help = "Overwrite existing files")]
    pub overwrite: bool,

    #[arg(short = 'q', long = "quiet", help = "Suppress non-essential output")]
    pub quiet: bool,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(name = "ARCHIVE", required_unless_present = "version")]
    pub archive: Option<String>,
}
