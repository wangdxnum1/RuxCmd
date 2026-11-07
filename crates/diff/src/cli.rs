use clap::{Arg, ArgAction, Command};

#[derive(Debug)]
pub struct Cli {
    pub version: bool,
    pub unified: bool,
    pub ignore_case: bool,
    pub ignore_whitespace: bool,
    pub ignore_blank_lines: bool,
    pub recursive: bool,
    /// Flag tokens echoed into directory-walk headers exactly as typed
    /// (`diff -ru ...`, `diff -B -i -r ...`, `diff --unified --recursive ...`).
    pub header_flags: Vec<String>,
    pub file1: Option<String>,
    pub file2: Option<String>,
}

pub fn parse_args() -> Cli {
    let matches = Command::new("diff")
        .version("0.1.0")
        .about("Compare two files line by line")
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::SetTrue)
                .help("Show version"),
        )
        .arg(
            Arg::new("unified")
                .short('u')
                .long("unified")
                .action(ArgAction::SetTrue)
                .help("Output unified diff format"),
        )
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .action(ArgAction::SetTrue)
                .help("Ignore case differences"),
        )
        .arg(
            Arg::new("ignore-whitespace")
                .short('b')
                .long("ignore-whitespace")
                .action(ArgAction::SetTrue)
                .help("Ignore whitespace differences"),
        )
        .arg(
            Arg::new("ignore-blank-lines")
                .short('B')
                .long("ignore-blank-lines")
                .action(ArgAction::SetTrue)
                .help("Ignore blank lines"),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("Recursively compare any subdirectories found"),
        )
        .arg(Arg::new("file1").help("First file to compare ('-' reads stdin)"))
        .arg(Arg::new("file2").help("Second file to compare ('-' reads stdin)"))
        .get_matches();

    // GNU echoes the comparison options in the walk header exactly as they
    // were typed (`-ru`, `-B -i -r`, `--unified --recursive`), so collect
    // the raw tokens from argv.  Anything that is not a comparison flag
    // (operands, `-`, `--`) ends the echo.
    const LONGS: [&str; 5] = [
        "--unified",
        "--recursive",
        "--ignore-case",
        "--ignore-whitespace",
        "--ignore-blank-lines",
    ];
    const SHORTS: [char; 5] = ['u', 'i', 'b', 'B', 'r'];
    let mut header_flags: Vec<String> = Vec::new();
    for token in std::env::args().skip(1) {
        if token == "--" || !token.starts_with('-') || token == "-" {
            break;
        }
        if let Some(long) = LONGS.iter().find(|l| token == **l) {
            header_flags.push((*long).to_string());
        } else if let Some(body) = token.strip_prefix('-') {
            if !body.is_empty() && body.chars().all(|c| SHORTS.contains(&c)) {
                header_flags.push(token);
            }
        }
    }

    Cli {
        version: matches.get_flag("version"),
        unified: matches.get_flag("unified"),
        ignore_case: matches.get_flag("ignore-case"),
        ignore_whitespace: matches.get_flag("ignore-whitespace"),
        ignore_blank_lines: matches.get_flag("ignore-blank-lines"),
        recursive: matches.get_flag("recursive"),
        header_flags,
        file1: matches.get_one::<String>("file1").cloned(),
        file2: matches.get_one::<String>("file2").cloned(),
    }
}
