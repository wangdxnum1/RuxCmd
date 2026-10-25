use clap::Parser;

/// Print the user names of users currently logged in — a Windows port of the Linux users command.
#[derive(Parser, Debug)]
#[command(name = "users", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print only the number of distinct users
    #[arg(short = 'q', long = "count")]
    pub count: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
