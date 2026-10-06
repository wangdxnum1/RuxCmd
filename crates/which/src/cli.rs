use clap::Parser;

/// Locate a command — a Windows port of the Linux which command.
#[derive(Parser, Debug)]
#[command(name = "which", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print all matching pathnames of each argument
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// The command(s) to locate
    #[arg(value_name = "COMMAND")]
    pub commands: Vec<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
