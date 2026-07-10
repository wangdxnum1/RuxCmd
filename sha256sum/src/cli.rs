use clap::{Command, Arg, ArgAction};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub check: bool,
    pub quiet: bool,
    pub files: Vec<PathBuf>,
}

pub fn parse_args() -> Args {
    let matches = Command::new("sha256sum")
        .version("0.1.0")
        .about("A Windows sha256sum clone — compute SHA256 hash of files")
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
                .help("Display version information and exit"),
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Read SHA256 sums from the FILEs and check them"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress all normal output"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("The file(s) to process")
                .num_args(0..),
        )
        .get_matches();

    Args {
        check: matches.get_flag("check"),
        quiet: matches.get_flag("quiet"),
        files: matches
            .get_many::<String>("files")
            .map(|v| v.map(|s| PathBuf::from(s)).collect())
            .unwrap_or_default(),
    }
}
