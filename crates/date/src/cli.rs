use clap::Parser;

/// Display or set date and time — a Windows port of the Linux date command.
#[derive(Parser, Debug)]
#[command(name = "date", version, about, disable_version_flag = true)]
pub struct Args {
    /// Display the given date string instead of the current time
    /// (@EPOCH, YYYY-MM-DD, or YYYY-MM-DDTHH:MM[:SS]; empty means now)
    #[arg(short = 'd', long = "date", value_name = "DATE")]
    pub date_string: Option<String>,

    /// Set the system date and time to the given value
    /// (same formats as --date; requires administrator privilege)
    #[arg(short = 's', long = "set", value_name = "DATE")]
    pub set_time: Option<String>,

    /// Display the resolved time as seconds since the Unix epoch
    #[arg(long = "seconds")]
    pub seconds: bool,

    /// Display the resolved time as nanoseconds since the Unix epoch
    #[arg(long = "nanoseconds")]
    pub nanoseconds: bool,

    /// Use UTC time instead of local time
    #[arg(short = 'u', long = "utc")]
    pub utc: bool,

    /// Date to display (like --date), or +FORMAT controlling the output format
    #[arg(value_name = "DATE|+FORMAT")]
    pub operand: Option<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
