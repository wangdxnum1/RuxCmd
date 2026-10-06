use clap::Parser;

/// Print information about users who are currently logged in — a Windows port of the Linux who command.
#[derive(Parser, Debug)]
#[command(name = "who", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print all available information
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Print system boot time
    #[arg(short = 'b', long = "boot")]
    pub boot_time: bool,

    /// Print dead processes
    #[arg(short = 'd', long = "dead")]
    pub dead: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
