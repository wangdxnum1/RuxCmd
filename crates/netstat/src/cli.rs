use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "netstat",
    version,
    about = "Print network connections",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'a', long = "all", action = clap::ArgAction::SetTrue)]
    pub all: bool,

    #[arg(short = 'n', long = "numeric", action = clap::ArgAction::SetTrue, help = "Numeric addresses and ports (always on, declared)")]
    pub numeric: bool,

    #[arg(
        short = 'o',
        long = "own-pid",
        help = "Include the owning process ID column"
    )]
    pub own_pid: bool,

    #[arg(short = 'p', long = "protocol")]
    pub protocol: Option<String>,
}
