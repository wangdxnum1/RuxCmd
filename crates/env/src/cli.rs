use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "env",
    version,
    about = "Print the environment or run a command with a modified environment",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(
        short = 'i',
        long = "ignore-environment",
        help = "Start with an empty environment"
    )]
    pub ignore_environment: bool,

    #[arg(
        short = 'u',
        long = "unset",
        value_name = "NAME",
        help = "Remove a variable from the environment",
        action = clap::ArgAction::Append
    )]
    pub unset: Vec<String>,

    #[arg(short = 'v', long = "verbose", help = "Verbose output")]
    pub verbose: bool,

    #[arg(short = 'V', long = "version", help = "Print version information", action = clap::ArgAction::Version)]
    pub version: (),

    /// Leading NAME=VALUE assignments, then the command to run; with no
    /// command the resulting environment is printed
    #[arg(
        value_name = "NAME=VALUE or COMMAND",
        num_args = 0..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<String>,
}
