use clap::Parser;

#[derive(Parser)]
#[command(name = "uptime", about = "显示系统运行时间")]
#[command(arg_required_else_help = false)]
pub struct Cli {
    #[arg(short, long, help = "以人类可读的格式显示")]
    pub pretty: bool,

    #[arg(short = 'v', long, help = "显示版本信息")]
    pub version: bool,
}

pub const VERSION: &str = "0.1.0";
