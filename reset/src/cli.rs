use clap::Parser;

/// Reset terminal state — a Windows port of the Linux reset command.
#[derive(Parser, Debug)]
#[command(name = "reset", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}