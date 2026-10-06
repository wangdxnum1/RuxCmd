use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gunzip", version = "0.1.0", about = "解压 gzip 文件")]
pub struct Cli {
    #[arg(short = 'k', long = "keep", help = "保留原始压缩文件")]
    pub keep: bool,

    #[arg(short = 'f', long = "force", help = "强制覆盖已存在的目标文件")]
    pub force: bool,

    #[arg(short = 'v', long = "verbose", help = "显示详细信息")]
    pub verbose: bool,

    #[arg(help = "要解压的 .gz 文件")]
    pub file: String,
}
