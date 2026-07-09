use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "tr",
    version = "0.1.0",
    about = "字符转换工具",
    disable_version_flag = true,
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "显示版本信息")]
    pub version: bool,

    #[arg(short = 'd', long = "delete", help = "删除 SET1 中的字符")]
    pub delete: bool,

    #[arg(short = 's', long = "squeeze", help = "压缩重复字符")]
    pub squeeze: bool,

    #[arg(short = 'c', long = "complement", help = "取 SET1 的补集")]
    pub complement: bool,

    #[arg(help = "第一个字符集")]
    pub set1: Option<String>,

    #[arg(help = "第二个字符集（用于替换模式）")]
    pub set2: Option<String>,
}