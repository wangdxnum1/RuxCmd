use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Human,
}

#[derive(Parser, Debug)]
#[command(
    name = "free",
    version,
    about,
    disable_version_flag = true,
    disable_help_flag = true
)]
/// Help stays a long-only flag: -h keeps its GNU meaning in this command.
pub struct Args {
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    #[arg(short = 'b', long = "bytes", conflicts_with_all = ["kilobytes", "megabytes", "gigabytes", "human_readable"])]
    pub bytes: bool,

    #[arg(short = 'k', long = "kilobytes", conflicts_with_all = ["bytes", "megabytes", "gigabytes", "human_readable"])]
    pub kilobytes: bool,

    #[arg(short = 'm', long = "megabytes", conflicts_with_all = ["bytes", "kilobytes", "gigabytes", "human_readable"])]
    pub megabytes: bool,

    #[arg(short = 'g', long = "gigabytes", conflicts_with_all = ["bytes", "kilobytes", "megabytes", "human_readable"])]
    pub gigabytes: bool,

    #[arg(short = 'h', long = "human-readable", conflicts_with_all = ["bytes", "kilobytes", "megabytes", "gigabytes"])]
    pub human_readable: bool,

    #[arg(
        short = 't',
        long = "total",
        help = "Show a line with Mem + Swap totals"
    )]
    pub total: bool,

    #[arg(
        short = 's',
        long = "seconds",
        value_name = "N",
        help = "Repeat every N seconds (with -c, up to COUNT times)"
    )]
    pub seconds: Option<u64>,

    #[arg(short = 'c', long = "count", value_name = "N", requires = "seconds")]
    pub count: Option<u64>,

    #[arg(
        short = 'w',
        long = "wide",
        help = "Wide output: show the available column instead of shared/buff-cache placeholders"
    )]
    pub wide: bool,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}

impl Args {
    pub fn get_unit(&self) -> Unit {
        if self.bytes {
            Unit::Bytes
        } else if self.kilobytes {
            Unit::Kilobytes
        } else if self.megabytes {
            Unit::Megabytes
        } else if self.gigabytes {
            Unit::Gigabytes
        } else if self.human_readable {
            Unit::Human
        } else {
            Unit::Kilobytes
        }
    }
}
