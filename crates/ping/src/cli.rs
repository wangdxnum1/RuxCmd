use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "ping",
    version,
    about = "Send ICMP echo requests",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'c', long = "count", default_value = "4")]
    pub count: usize,

    #[arg(short = 't', long = "timeout", default_value = "4")]
    pub timeout: u64,

    #[arg(short = 's', long = "size", default_value = "32")]
    pub size: usize,

    #[arg(value_name = "HOST")]
    pub host: Option<String>,
}
