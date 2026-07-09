use clap::Parser;

/// Display a line of text — a Windows port of the Linux echo command.
#[derive(Parser, Debug)]
#[command(name = "echo", version, about, disable_version_flag = true)]
pub struct Args {
    /// Do not output the trailing newline
    #[arg(short = 'n')]
    pub no_newline: bool,

    /// Enable interpretation of backslash escapes
    #[arg(short = 'e')]
    pub enable_escapes: bool,

    /// Disable interpretation of backslash escapes (default)
    #[arg(short = 'E')]
    pub disable_escapes: bool,

    /// The text to display
    #[arg(value_name = "STRING")]
    pub strings: Vec<String>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}