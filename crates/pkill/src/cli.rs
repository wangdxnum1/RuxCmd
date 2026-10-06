use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pkill", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'f', long = "full", help = "Match full command line")]
    pub full: bool,

    #[arg(
        short = 's',
        long = "signal",
        help = "Signal to send (ignored on Windows)"
    )]
    pub signal: Option<String>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "PATTERN")]
    pub pattern: String,
}
