use clap::Parser;
use std::path::PathBuf;

/// Create empty files or update file timestamps — a Windows port of the Linux touch command.
#[derive(Parser, Debug)]
#[command(
    name = "touch",
    version,
    about,
    disable_version_flag = true,
    after_help = "-d/--date supports only @SECONDS and 'YYYY-MM-DD[THH:MM[:SS]]' or\n'YYYY-MM-DD HH:MM[:SS]' (local time); relative date expressions such as\n'1 day ago' are not supported and fail with `invalid date format`."
)]
pub struct Args {
    /// Do not create any files
    #[arg(short = 'c', long = "no-create")]
    pub no_create: bool,

    /// Change only the access time
    #[arg(short = 'a')]
    pub atime: bool,

    /// Change only the modification time
    #[arg(short = 'm')]
    pub mtime: bool,

    /// Use this file's times instead of the current time
    #[arg(short = 'r', long = "reference", value_name = "FILE")]
    pub reference: Option<PathBuf>,

    /// Use [[CC]YY]MMDDhhmm[.ss] (local time) instead of the current time
    #[arg(short = 't', value_name = "STAMP")]
    pub stamp: Option<String>,

    /// Use @SECONDS or an ISO date/time (local time) instead of the current time
    #[arg(short = 'd', long = "date", value_name = "DATE")]
    pub date: Option<String>,

    /// The file(s) to touch
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Show version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
