use clap::{Arg, Command};

#[derive(Debug, Clone)]
pub struct Args {
    pub max_args: usize,
    pub interactive: bool,
    pub verbose: bool,
    pub replace: String,
    pub replace_set: bool,
    pub command: Vec<String>,
}

impl Args {
    pub fn parse() -> Self {
        let matches = Command::new("xargs")
            .version("0.1.0")
            .about("Build and execute command lines from standard input")
            .disable_version_flag(true)
            .arg(
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .help("Print version information and exit")
                    .action(clap::ArgAction::Version),
            )
            .arg(
                Arg::new("max-args")
                    .short('n')
                    .long("max-args")
                    .help("Use at most max-args arguments per command line")
                    .default_value("0")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                Arg::new("interactive")
                    .short('p')
                    .long("interactive")
                    .help("Prompt before running each command")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("verbose")
                    .short('t')
                    .long("verbose")
                    .help("Print the command line on the standard error output before executing it")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("replace-short")
                    .short('i')
                    .help("Same as --replace={}")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("replace")
                    .long("replace")
                    .help("Replace occurrences of replace-str in the initial arguments with names read from standard input"),
            )
            .arg(
                Arg::new("command")
                    .help("The command to execute with optional initial arguments")
                    .value_name("COMMAND")
                    .num_args(0..),
            )
            .get_matches();

        let replace_set = matches.get_flag("replace-short") || matches.contains_id("replace");
        let replace = if matches.get_flag("replace-short") {
            "{}".to_string()
        } else {
            matches.get_one::<String>("replace").cloned().unwrap_or_else(|| "{}".to_string())
        };

        Args {
            max_args: matches.get_one::<usize>("max-args").copied().unwrap_or(0),
            interactive: matches.get_flag("interactive"),
            verbose: matches.get_flag("verbose"),
            replace,
            replace_set,
            command: matches.get_many::<String>("command").map(|v| v.cloned().collect()).unwrap_or_default(),
        }
    }
}
