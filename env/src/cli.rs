use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "env", version, about = "Display environment variables", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'i', long = "ignore-environment", help = "Start with an empty environment")]
    pub ignore_environment: bool,

    #[arg(short = 'u', long = "unset", help = "Unset a variable", action = clap::ArgAction::Append)]
    pub unset: Vec<String>,

    #[arg(short = 'v', long = "verbose", help = "Verbose output")]
    pub verbose: bool,

    #[arg(short = 'V', long = "version", help = "Print version information", action = clap::ArgAction::Version)]
    pub version: (),
}
