use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "patch",
    about = "Apply a diff patch to a file",
    version = "0.1.0",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", help = "Show version")]
    pub version: bool,

    #[arg(
        short = 'p',
        long = "strip",
        help = "Strip the smallest prefix containing num leading slashes"
    )]
    pub strip: Option<usize>,

    #[arg(
        short = 'f',
        long = "force",
        help = "Force apply the patch even if there are rejects"
    )]
    pub force: bool,

    #[arg(
        short = 'R',
        long = "reverse",
        help = "Reverse the patch (undo the changes)"
    )]
    pub reverse: bool,

    #[arg(help = "Patch file to apply")]
    pub patch_file: Option<String>,

    #[arg(help = "File to patch")]
    pub target_file: Option<String>,
}
