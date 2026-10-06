use clap::Parser;

/// Print the groups a user is in — a Windows port of the Linux groups command.
#[derive(Parser, Debug)]
#[command(name = "groups", version, about, disable_version_flag = true)]
pub struct Args {
    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Username to query groups for (default: current user)
    #[arg(name = "USER")]
    pub user: Option<String>,
}
