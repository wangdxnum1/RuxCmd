use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "time", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "COMMAND")]
    pub command: Vec<String>,
}