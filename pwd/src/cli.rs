use clap::Parser;

/// Print the full filename of the current working directory.
#[derive(Parser, Debug)]
#[command(name = "pwd", version, about, disable_version_flag = true)]
pub struct Args {
    /// Print the value of $PWD if it matches the current working directory
    #[arg(short = 'L', long = "logical", overrides_with = "physical")]
    pub logical: bool,

    /// Print the physical working directory (resolve all symlinks)
    #[arg(short = 'P', long = "physical", overrides_with = "logical")]
    pub physical: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}

impl Args {
    pub fn parse_or_exit() -> Self {
        Self::parse()
    }

    /// Returns true when physical mode is explicitly requested.
    /// Default is logical mode (matching GNU coreutils).
    pub fn is_physical(&self) -> bool {
        self.physical && !self.logical
    }
}
