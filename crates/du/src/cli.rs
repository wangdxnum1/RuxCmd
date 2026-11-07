use clap::parser::ValueSource;
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

/// One of the three GNU block-scale selectors.
#[derive(Debug)]
pub enum BlockScale {
    Kibi,
    Mebi,
    Size(String),
}

#[derive(Debug)]
pub struct Args {
    pub human_readable: bool,
    pub bytes: bool,
    pub apparent_size: bool,
    pub all: bool,
    pub summarize: bool,
    pub max_depth: Option<String>,
    pub one_file_system: bool,
    pub exclude: Vec<String>,
    pub total: bool,
    /// NUL-separated operand list from FILE ('-' = stdin).
    pub files0_from: Option<String>,
    /// Some(field) once --time (or --time=WORD) is given.
    pub time: Option<String>,
    pub time_style: Option<String>,
    /// -k/-m/--block-size candidates as (command-line position, spec); main
    /// validates all of them and the last position wins (GNU).
    pub block_scale_specs: Vec<(usize, BlockScale)>,
    pub paths: Vec<PathBuf>,
}

pub fn parse_args() -> Args {
    let matches = Command::new("du")
        .version("0.1.0")
        .about("Estimate file space usage — a Windows port of the Linux du command")
        .disable_version_flag(true)
        .disable_help_flag(true)
        // Help stays a long-only flag: -h keeps its GNU meaning here.
        .arg(
            Arg::new("help")
                .long("help")
                .action(ArgAction::Help)
                .help("Print help"),
        )
        .arg(
            Arg::new("human-readable")
                .short('h')
                .long("human-readable")
                .action(ArgAction::SetTrue)
                .help("Show human-readable sizes (1.0K, 4.9K, 10K, 1.0M)"),
        )
        .arg(
            Arg::new("bytes")
                .short('b')
                .long("bytes")
                .action(ArgAction::SetTrue)
                .help("Print sizes in bytes instead of 1K blocks (implies --apparent-size)"),
        )
        .arg(
            Arg::new("apparent-size")
                .long("apparent-size")
                .action(ArgAction::SetTrue)
                .help("Count apparent (logical) sizes instead of allocated disk space"),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Write counts for all files, not just directories"),
        )
        .arg(
            Arg::new("summarize")
                .short('s')
                .long("summarize")
                .action(ArgAction::SetTrue)
                .help("Show totals for all arguments"),
        )
        .arg(
            Arg::new("max-depth")
                .short('d')
                .long("max-depth")
                .value_name("N")
                .help("Print only entries N or fewer levels below the arguments"),
        )
        .arg(
            Arg::new("one-file-system")
                .short('x')
                .long("one-file-system")
                .action(ArgAction::SetTrue)
                .help("Skip directories on different file systems"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .value_name("PATTERN")
                .action(ArgAction::Append)
                .help("Exclude files that match PATTERN (repeatable)"),
        )
        .arg(
            Arg::new("total")
                .short('c')
                .long("total")
                .action(ArgAction::SetTrue)
                .help("Show grand total"),
        )
        .arg(
            Arg::new("files0-from")
                .long("files0-from")
                .value_name("FILE")
                .help("Read NUL-separated operand names from FILE ('-' = stdin)"),
        )
        .arg(
            Arg::new("time")
                .long("time")
                .require_equals(true)
                .num_args(0..=1)
                .default_missing_value("modification")
                .value_name("WORD")
                .help("Show the time of the last modification, or of any of atime/access/use, ctime/status"),
        )
        .arg(
            Arg::new("time-style")
                .long("time-style")
                .value_name("STYLE")
                .help("Time format for --time: full-iso, long-iso, iso, or +FORMAT"),
        )
        .arg(
            Arg::new("block-size")
                .long("block-size")
                .value_name("SIZE")
                .help("Scale sizes by SIZE before rounding (GNU suffixes: K, KB, M, MB, ...)"),
        )
        .arg(
            Arg::new("kibiblocks")
                .short('k')
                .action(ArgAction::SetTrue)
                .help("Count in 1024-byte blocks"),
        )
        .arg(
            Arg::new("mebiblocks")
                .short('m')
                .action(ArgAction::SetTrue)
                .help("Count in 1048576-byte blocks"),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
                .help("Show version information"),
        )
        .arg(Arg::new("paths").value_name("PATH").num_args(0..))
        .get_matches();

    // GNU -k/-m/--block-size: the last one on the command line wins; main
    // resolves that from the args, guarded by value_source so the clap
    // defaults for absent flags never influence the choice (see sha*sum).
    Args {
        human_readable: matches.get_flag("human-readable"),
        bytes: matches.get_flag("bytes"),
        apparent_size: matches.get_flag("apparent-size"),
        all: matches.get_flag("all"),
        summarize: matches.get_flag("summarize"),
        max_depth: matches.get_one::<String>("max-depth").cloned(),
        one_file_system: matches.get_flag("one-file-system"),
        exclude: matches
            .get_many::<String>("exclude")
            .map(|v| v.cloned().collect())
            .unwrap_or_default(),
        total: matches.get_flag("total"),
        files0_from: matches.get_one::<String>("files0-from").cloned(),
        time: if matches.value_source("time") == Some(ValueSource::CommandLine) {
            Some(
                matches
                    .get_one::<String>("time")
                    .cloned()
                    .unwrap_or_else(|| "modification".to_string()),
            )
        } else {
            None
        },
        time_style: if matches.value_source("time-style") == Some(ValueSource::CommandLine) {
            matches.get_one::<String>("time-style").cloned()
        } else {
            None
        },
        block_scale_specs: {
            let mut specs: Vec<(usize, BlockScale)> = Vec::new();
            if matches.value_source("kibiblocks") == Some(ValueSource::CommandLine)
                && let Some(pos) = matches.indices_of("kibiblocks").and_then(Iterator::max)
            {
                specs.push((pos, BlockScale::Kibi));
            }
            if matches.value_source("mebiblocks") == Some(ValueSource::CommandLine)
                && let Some(pos) = matches.indices_of("mebiblocks").and_then(Iterator::max)
            {
                specs.push((pos, BlockScale::Mebi));
            }
            if matches.value_source("block-size") == Some(ValueSource::CommandLine)
                && let Some(pos) = matches.indices_of("block-size").and_then(Iterator::max)
            {
                specs.push((
                    pos,
                    BlockScale::Size(
                        matches
                            .get_one::<String>("block-size")
                            .cloned()
                            .unwrap_or_default(),
                    ),
                ));
            }
            specs
        },
        paths: matches
            .get_many::<String>("paths")
            .map(|v| v.map(PathBuf::from).collect())
            .unwrap_or_default(),
    }
}
