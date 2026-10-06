use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None, disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Display version information")]
    pub version: bool,

    #[arg(short = 's', long, help = "Remove a trailing suffix")]
    pub suffix: Option<String>,

    #[arg(short = 'a', long, help = "Process multiple arguments")]
    pub multiple: bool,

    #[arg(name = "NAME", help = "File path(s) to process")]
    pub names: Vec<String>,
}
