use clap::Parser;

/// Repeat the same string indefinitely — a Windows port of the Linux yes command.
#[derive(Parser, Debug)]
#[command(name = "yes", version, about, disable_version_flag = true)]
pub struct Args {
    /// The string to repeat (default: "y")
    #[arg(value_name = "STRING")]
    pub string: Option<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
