use clap::Parser;

/// Send signal to processes — a Windows port of the Linux kill command.
#[derive(Parser, Debug)]
#[command(name = "kill", version, about, disable_version_flag = true)]
pub struct Args {
    /// Signal to send (default: TERM/15)
    #[arg(short = 's', long = "signal")]
    pub signal: Option<String>,

    /// List all available signals
    #[arg(short = 'l', long = "list")]
    pub list_signals: bool,

    /// Process IDs to send signal to
    #[arg(value_name = "PID")]
    pub pids: Vec<i32>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
