use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "seq", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 's', long = "separator", default_value = "\n")]
    pub separator: String,

    #[arg(short = 'w', long = "equal-width")]
    pub equal_width: bool,

    #[arg(short = 'f', long = "format")]
    pub format: Option<String>,

    #[arg(value_name = "NUMBER")]
    pub numbers: Vec<String>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
