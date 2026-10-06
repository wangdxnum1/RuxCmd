use clap::Parser;

/// Clear the terminal screen — a Windows port of the Linux clear command.
#[derive(Parser, Debug)]
#[command(name = "clear", version, about, disable_version_flag = true)]
pub struct Args {
    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
