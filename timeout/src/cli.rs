use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "timeout", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'k', long = "kill-after", help = "Send kill signal after timeout")]
    pub kill_after: Option<String>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "DURATION")]
    pub duration: String,

    #[arg(value_name = "COMMAND")]
    pub command: Vec<String>,
}