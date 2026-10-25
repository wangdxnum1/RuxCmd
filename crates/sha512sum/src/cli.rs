use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub zero: bool,
    pub check: bool,
    pub quiet: bool,
    pub status: bool,
    pub tag: bool,
    pub files: Vec<PathBuf>,
}

pub fn parse_args() -> Args {
    let matches = Command::new("sha512sum")
        .version("0.1.0")
        .about("A Windows sha512sum clone — compute SHA512 hash of files")
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
            Arg::new("check")
                .short('c')
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Read SHA512 sums from the FILEs and check them"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress all normal output"),
        )
        .arg(
            Arg::new("status")
                .long("status")
                .action(ArgAction::SetTrue)
                .help("Don't output anything; the exit status indicates success or failure"),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .action(ArgAction::SetTrue)
                .help("Create a BSD-style checksum (\"SHA512 (file) = hash\")"),
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
        check: matches.get_flag("check"),
        quiet: matches.get_flag("quiet"),
        status: matches.get_flag("status"),
        tag: matches.get_flag("tag"),
        files: matches
            .get_many::<String>("files")
            .map(|v| v.map(PathBuf::from).collect())
            .unwrap_or_default(),
    }
}
