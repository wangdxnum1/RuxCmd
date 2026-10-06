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
#[command(name = "free", version, about, disable_version_flag = true)]
pub struct Args {
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
