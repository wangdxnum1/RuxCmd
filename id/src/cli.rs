use clap::Parser;

/// Print user and group information — a Windows port of the Linux id command.
#[derive(Parser, Debug)]
#[command(name = "id", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print the effective user ID
    #[arg(short = 'u', long = "user")]
    pub user: bool,

    /// Print the effective group ID
    #[arg(short = 'g', long = "group")]
    pub group: bool,

    /// Print all group IDs
    #[arg(short = 'G', long = "groups")]
    pub groups: bool,

    /// Print names instead of numeric IDs
    #[arg(short = 'n', long = "name")]
    pub name: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
