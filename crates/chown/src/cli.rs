use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "chown",
    version = "0.1.0",
    about = "更改文件所有者",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(long, help = "目标用户名")]
    pub user: Option<String>,

    #[arg(long, help = "目标用户组")]
    pub group: Option<String>,

    #[arg(short = 'R', long, help = "递归更改目录下所有文件")]
    pub recursive: bool,

    #[arg(short = 'v', long, help = "详细输出")]
    pub verbose: bool,

    #[arg(short = 'V', long = "version", help = "显示版本信息", action = clap::ArgAction::Version)]
    pub version: (),

    #[arg(help = "目标文件或目录路径")]
    pub path: Vec<String>,
}
