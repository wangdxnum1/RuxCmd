use clap::Parser;

/// Display or set date and time — a Windows port of the Linux date command.
#[derive(Parser, Debug)]
#[command(name = "date", version, about)]
pub struct Args {
    /// Display date using the specified format
    #[arg(short = 'd', long = "date")]
    pub date_string: Option<String>,

    /// Set the system date and time
    #[arg(short = 's', long = "set")]
    pub set_time: Option<String>,

    /// Display the time as seconds since the Unix epoch
    #[arg(short = 's', long = "seconds")]
    pub seconds: bool,

    /// Display the time as nanoseconds since the Unix epoch
    #[arg(long = "nanoseconds")]
    pub nanoseconds: bool,

    /// Use UTC time instead of local time
    #[arg(short = 'u', long = "utc")]
    pub utc: bool,

    /// Format string
    #[arg(value_name = "FORMAT")]
    pub format: Option<String>,
}