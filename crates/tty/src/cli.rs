use clap::Parser;

/// Print terminal name — a Windows port of the Linux tty command.
#[derive(Parser, Debug)]
#[command(name = "tty", version, about, disable_version_flag = true)]
pub struct Args {
    /// Silent mode - do not print anything, just return exit code
    #[arg(short = 's', long = "silent")]
    pub silent: bool,

    /// Print information about all three standard streams
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
