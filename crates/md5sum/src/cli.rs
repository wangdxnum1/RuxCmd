use clap::parser::ValueSource;
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub zero: bool,
    pub check: bool,
    pub quiet: bool,
    pub status: bool,
    pub tag: bool,
    /// Resolved checksum mode: true = binary (`*` separator).  When both
    /// -b and -t appear, the last one on the command line wins (GNU).
    pub binary: bool,
    /// True when -b or -t appeared at all; both are meaningless with --check.
    pub mode_explicit: bool,
    pub files: Vec<PathBuf>,
}

pub fn parse_args() -> Args {
    let matches = Command::new("md5sum")
        .version("0.1.0")
        .about("Compute MD5 hash sums of files — a Windows port of the Linux md5sum command")
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
                .help("Read MD5 sums from the FILEs and check them"),
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
                .help("Create a BSD-style checksum (\"MD5 (file) = hash\")"),
        )
        .arg(
            Arg::new("binary")
                .short('b')
                .long("binary")
                .action(ArgAction::SetTrue)
                .help("Print the checksum with a star separator (binary mode)"),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .action(ArgAction::SetTrue)
                .help("Print the checksum with two-space separator (text mode)"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("The file(s) to process")
                .num_args(0..),
        )
        .get_matches();

    // GNU: when both -b and -t are given, the last one wins.  Flags absent
    // from the command line still carry a clap default index, so positions
    // are only compared for flags actually passed on the command line.
    let binary_given = matches.value_source("binary") == Some(ValueSource::CommandLine);
    let text_given = matches.value_source("text") == Some(ValueSource::CommandLine);
    let binary_last = matches.indices_of("binary").and_then(Iterator::max);
    let text_last = matches.indices_of("text").and_then(Iterator::max);
    let binary = match (binary_given, text_given) {
        (true, true) => binary_last.unwrap_or(0) > text_last.unwrap_or(0),
        (true, false) => true,
        _ => false,
    };
    Args {
        zero: matches.get_flag("zero"),
        check: matches.get_flag("check"),
        quiet: matches.get_flag("quiet"),
        status: matches.get_flag("status"),
        tag: matches.get_flag("tag"),
        binary,
        mode_explicit: binary_given || text_given,
        files: matches
            .get_many::<String>("files")
            .map(|v| v.map(PathBuf::from).collect())
            .unwrap_or_default(),
    }
}
