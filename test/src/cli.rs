use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "test", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'e', name = "FILE", conflicts_with_all = ["file", "dir", "size", "operand"])]
    pub exists: Option<String>,

    #[arg(short = 'f', name = "FILE", conflicts_with_all = ["exists", "dir", "size", "operand"])]
    pub file: Option<String>,

    #[arg(short = 'd', name = "FILE", conflicts_with_all = ["exists", "file", "size", "operand"])]
    pub dir: Option<String>,

    #[arg(short = 's', name = "FILE", conflicts_with_all = ["exists", "file", "dir", "operand"])]
    pub size: Option<String>,

    #[arg(name = "OPERAND", num_args = 0..=3)]
    pub operand: Vec<String>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
