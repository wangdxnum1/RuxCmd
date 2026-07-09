use clap::Parser;

/// Locate a command — a Windows port of the Linux which command.
#[derive(Parser, Debug)]
#[command(name = "which", version, about)]
pub struct Args {
    /// Print all matching pathnames of each argument
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// The command(s) to locate
    #[arg(value_name = "COMMAND")]
    pub commands: Vec<String>,
}