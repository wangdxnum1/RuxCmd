use clap::Parser;

#[derive(Parser)]
#[command(name = "w", about = "显示用户活动", disable_help_flag = true, disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'h', long = "header", help = "显示头部信息")]
    pub header: bool,

    #[arg(short = 'u', long = "ignore-username", help = "忽略用户名")]
    pub ignore_username: bool,

    #[arg(short = 'v', long = "version", help = "显示版本信息")]
    pub version: bool,

    #[arg(long = "help", help = "显示帮助信息")]
    pub help: bool,
}
