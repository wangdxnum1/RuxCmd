use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "whereis", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "COMMAND")]
    pub commands: Vec<String>,
}
