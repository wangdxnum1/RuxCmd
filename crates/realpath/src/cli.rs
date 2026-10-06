use clap::Parser;

/// Print the canonicalized absolute pathname.
#[derive(Parser, Debug)]
#[command(name = "realpath", version, about, disable_version_flag = true)]
pub struct Args {
    /// Strip the specified number of leading path components
    #[arg(short = 's', long = "strip-components", default_value_t = 0)]
    pub strip_components: usize,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Path to resolve
    #[arg(default_value = ".")]
    pub path: String,
}

impl Args {
    pub fn parse_or_exit() -> Self {
        Self::parse()
    }
}
