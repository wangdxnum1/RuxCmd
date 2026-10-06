use clap::Parser;

/// Strip non-directory suffix from file name.
#[derive(Parser, Debug)]
#[command(name = "dirname", version, about, disable_version_flag = true)]
pub struct Args {
    /// File path
    #[arg(required = true)]
    pub path: Vec<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}

impl Args {
    pub fn parse_or_exit() -> Self {
        Self::parse()
    }
}
