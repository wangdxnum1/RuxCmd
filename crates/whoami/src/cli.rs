use clap::Parser;

/// Print effective userid — a Windows port of the Linux whoami command.
#[derive(Parser, Debug)]
#[command(name = "whoami", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print the hostname
    #[arg(short = 'h', long = "hostname")]
    pub hostname: bool,

    /// Print the username
    #[arg(short = 'u', long = "username")]
    pub username: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
