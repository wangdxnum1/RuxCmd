use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "factor", version, about = "Print prime factors of NUMBER(s)", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(value_name = "NUMBER")]
    pub numbers: Vec<String>,
}
