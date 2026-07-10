use clap::Parser;

/// Display calendar — a Windows port of the Linux cal command.
#[derive(Parser, Debug)]
#[command(name = "cal", version, about, disable_version_flag = true)]
pub struct Args {
    /// Display three months (previous, current, next)
    #[arg(short = '3')]
    pub three_months: bool,

    /// Display the entire year
    #[arg(short = 'y')]
    pub year: bool,

    /// Display Julian dates
    #[arg(short = 'j')]
    pub julian: bool,

    /// Month to display (1-12)
    #[arg(value_name = "MONTH")]
    pub month: Option<u32>,

    /// Year to display
    #[arg(value_name = "YEAR")]
    pub year_arg: Option<i32>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
