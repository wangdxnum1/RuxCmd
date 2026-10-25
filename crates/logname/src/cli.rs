use clap::Parser;

/// Print the current login name — a Windows port of the Linux logname command.
#[derive(Parser, Debug)]
#[command(name = "logname", version, about, disable_version_flag = true)]
pub struct Args {
    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Accepted for GNU parity but rejected: logname takes no operands
    #[arg(value_name = "USER", hide = true)]
    pub operands: Vec<String>,
}
