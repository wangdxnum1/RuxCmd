use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "nice", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(
        short = 'n',
        long = "adjustment",
        default_value = "10",
        help = "Priority adjustment"
    )]
    pub adjustment: i32,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(value_name = "COMMAND")]
    pub command: Vec<String>,
}
