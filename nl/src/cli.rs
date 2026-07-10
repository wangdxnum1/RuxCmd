use clap::{Arg, Command};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub body_numbering: BodyNumbering,
    pub separator: String,
    pub number_width: usize,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub enum BodyNumbering {
    All,
    NonBlank,
}

impl std::str::FromStr for BodyNumbering {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" | "a" => Ok(BodyNumbering::All),
            "nonblank" | "n" => Ok(BodyNumbering::NonBlank),
            _ => Err(format!("Invalid body numbering: {}", s)),
        }
    }
}

pub fn parse_args() -> Args {
    let matches = Command::new("nl")
        .about("Number lines of files — a Windows port of the Linux nl command.")
        .version("0.1.0")
        .disable_version_flag(true)
        .arg(
            Arg::new("body-numbering")
                .short('b')
                .long("body-numbering")
                .default_value("all")
                .help("Number all lines (body numbering)"),
        )
        .arg(
            Arg::new("separator")
                .short('s')
                .long("separator")
                .default_value("\t")
                .help("Use STRING as line number separator"),
        )
        .arg(
            Arg::new("number-width")
                .short('w')
                .long("number-width")
                .default_value("6")
                .help("Use NUMBER columns for line numbers"),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(clap::ArgAction::SetTrue)
                .help("Print version information"),
        )
        .arg(
            Arg::new("FILE")
                .value_name("FILE")
                .help("The file(s) to number")
                .num_args(0..),
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("nl 0.1.0");
        std::process::exit(0);
    }

    let body_numbering: BodyNumbering = matches
        .get_one::<String>("body-numbering")
        .unwrap()
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("nl: {}", e);
            std::process::exit(1);
        });

    let separator = matches.get_one::<String>("separator").unwrap().clone();

    let number_width: usize = matches
        .get_one::<String>("number-width")
        .unwrap()
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("nl: invalid number width: {}", e);
            std::process::exit(1);
        });

    let files: Vec<PathBuf> = matches
        .get_many::<String>("FILE")
        .map(|v| v.map(|s| PathBuf::from(s)).collect())
        .unwrap_or_default();

    Args {
        body_numbering,
        separator,
        number_width,
        files,
    }
}