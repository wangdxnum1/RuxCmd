use clap::Parser;

/// Return a failure exit code
#[derive(Parser, Debug)]
#[command(name = "false", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}