use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "tar", version = "0.1.0", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'c', long = "create", conflicts_with_all = ["extract", "list"])]
    pub create: bool,

    #[arg(short = 'x', long = "extract", conflicts_with_all = ["create", "list"])]
    pub extract: bool,

    #[arg(short = 't', long = "list", conflicts_with_all = ["create", "extract"])]
    pub list: bool,

    #[arg(short = 'f', long = "file", required = true)]
    pub file: String,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(short = 'z', long = "gzip")]
    pub gzip: bool,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(name = "FILES", required_unless_present_any = ["list", "extract"])]
    pub files: Vec<String>,
}
