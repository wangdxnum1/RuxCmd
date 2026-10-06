use clap::Parser;

/// Formatted output — a Windows port of the Linux printf command.
#[derive(Parser, Debug)]
#[command(name = "printf", version, about, disable_version_flag = true)]
pub struct Args {
    /// Format string
    #[arg(value_name = "FORMAT")]
    pub format: String,

    /// Arguments for format specifiers
    #[arg(value_name = "ARG")]
    pub args: Vec<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
