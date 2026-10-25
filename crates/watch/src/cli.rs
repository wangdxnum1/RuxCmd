use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "watch",
    version,
    about = "Run COMMAND repeatedly, showing its output full screen, Linux-style",
    disable_version_flag = true
)]
pub struct Args {
    /// Seconds to wait between updates; fractional values are allowed and the
    /// interval is clamped to 0.1..2678400 seconds like procps watch
    #[arg(
        short = 'n',
        long = "interval",
        value_name = "SECONDS",
        default_value_t = String::from("2")
    )]
    pub interval: String,

    /// Highlight the lines whose output changed since the previous update
    #[arg(short = 'd', long = "differences")]
    pub differences: bool,

    /// Exit when the output of the command changes between updates
    #[arg(short = 'g', long = "chgexit")]
    pub chgexit: bool,

    /// Exit when the command exits with a non-zero status
    #[arg(short = 'e', long = "errexit")]
    pub errexit: bool,

    /// Execute the command directly instead of through a shell (accepted for
    /// GNU compatibility only: the command is never run through a shell here)
    #[arg(short = 'x', long = "exec")]
    pub exec: bool,

    /// Turn off the header line and the rule below it
    #[arg(short = 't', long = "no-title")]
    pub no_title: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Command to run; everything after COMMAND is passed through unchanged,
    /// even when it starts with '-'
    #[arg(
        value_name = "COMMAND",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<String>,
}
