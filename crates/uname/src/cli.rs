use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "uname",
    author,
    version,
    about = "Print system information",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'a', long = "all", help = "Print all system information")]
    pub all: bool,

    #[arg(short = 's', long = "kernel-name", help = "Print kernel name")]
    pub kernel_name: bool,

    #[arg(short = 'n', long = "nodename", help = "Print network node hostname")]
    pub nodename: bool,

    #[arg(short = 'r', long = "kernel-release", help = "Print kernel release")]
    pub kernel_release: bool,

    #[arg(short = 'v', long = "kernel-version", help = "Print kernel version")]
    pub kernel_version: bool,

    #[arg(short = 'm', long = "machine", help = "Print machine hardware name")]
    pub machine: bool,

    #[arg(
        short = 'o',
        long = "operating-system",
        help = "Print operating system"
    )]
    pub operating_system: bool,

    #[arg(short = 'V', long = "version", help = "Print version information", action = clap::ArgAction::Version)]
    pub version: (),
}

impl Cli {
    pub fn has_no_flags(&self) -> bool {
        !self.all
            && !self.kernel_name
            && !self.nodename
            && !self.kernel_release
            && !self.kernel_version
            && !self.machine
            && !self.operating_system
    }
}
