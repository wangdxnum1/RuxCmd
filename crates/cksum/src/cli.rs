use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub zero: bool,
    pub algorithm: String,
    pub files: Vec<PathBuf>,
}

pub fn parse_args() -> Args {
    let matches = Command::new("cksum")
        .version("0.1.0")
        .about("A Windows cksum clone — compute POSIX CRC32 checksum of files")
        .disable_version_flag(true)
        .arg(
            Arg::new("zero")
                .short('z')
                .long("zero")
                .action(ArgAction::SetTrue)
                .help("End output records with NUL and do not escape file names"),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
                .help("Display version information and exit"),
        )
        .arg(
            Arg::new("algorithm")
                .short('a')
                .long("algorithm")
                .value_name("TYPE")
                .default_value("crc")
                .help("Select the digest algorithm (only crc is supported)"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("The file(s) to process")
                .num_args(0..),
        )
        .get_matches();

    Args {
        zero: matches.get_flag("zero"),
        algorithm: matches
            .get_one::<String>("algorithm")
            .cloned()
            .unwrap_or_else(|| "crc".to_string()),
        files: matches
            .get_many::<String>("files")
            .map(|v| v.map(PathBuf::from).collect())
            .unwrap_or_default(),
    }
}
