use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "whereis", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(short = 'b', long = "binaries", help = "Search only for binaries")]
    pub binaries: bool,

    #[arg(short = 'm', long = "manuals", help = "Search only for manuals")]
    pub manuals: bool,

    #[arg(short = 's', long = "sources", help = "Search only for sources")]
    pub sources: bool,

    #[arg(value_name = "COMMAND")]
    pub commands: Vec<String>,
}
