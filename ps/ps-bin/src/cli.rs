use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueHint};

const ABOUT: &str = "Show running processes (Linux-style ps for Windows)";
const LONG_ABOUT: &str = "ps — Linux-style process viewer ported to Windows.

Backed by Win32 (EnumProcesses + Toolhelp32 + PEB). Column names and flags
mirror procps-ng `ps` so muscle memory from Linux works on Windows. Values
that cannot be retrieved (denied PEB, missing token, etc.) are rendered as
'?' rather than aborting the listing.

Project home: https://github.com/your-org/ps";

const EXAMPLES: &str = "\
EXAMPLES:
  ps                          # default columns: PID TTY TIME COMM
  ps -e                        # every process
  ps -ef                       # full format
  ps -o pid,user,pcpu,comm     # custom columns
  ps -p 1234,5678              # filter by PID
  ps -u Tim                    # filter by user
  ps --sort=-pcpu              # sort by %CPU descending
  ps --forest --ascii          # process tree with ASCII connectors
  ps -ef --no-cmdline          # skip cmdline lookup (faster on busy systems)
  ps --list-columns            # list all available column names";

#[derive(Debug, Parser)]
#[command(
    name = "ps",
    version,
    author,
    about = ABOUT,
    long_about = LONG_ABOUT,
    disable_help_subcommand = true,
    disable_version_flag = true,
    next_help_heading = "Process selection",
    help_template = "{name} {version}\n{author-with-newline}{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}\n{after-help}",
    after_help = EXAMPLES,
)]
pub struct Cli {
    /// Select all processes
    #[arg(short = 'e', long = "everyone", help_heading = "Process selection")]
    pub select_all: bool,

    /// Select all processes (Linux `ps -A` style)
    #[arg(short = 'A', help_heading = "Process selection")]
    pub select_all_alt: bool,

    /// Select all processes with a tty
    #[arg(short = 'a', help_heading = "Process selection")]
    pub select_all_tty: bool,

    /// Include processes with no controlling terminal
    #[arg(short = 'x', help_heading = "Process selection")]
    pub include_no_tty: bool,

    /// Full-format listing (-f)
    #[arg(short = 'f', long = "full", help_heading = "Output format")]
    pub full: bool,

    /// Extra full-format listing
    #[arg(short = 'F', long = "extra-full", help_heading = "Output format")]
    pub extra_full: bool,

    /// Long format
    #[arg(short = 'l', long = "long", help_heading = "Output format")]
    pub long: bool,

    /// Jobs format
    #[arg(short = 'j', long = "jobs", help_heading = "Output format")]
    pub jobs: bool,

    /// Define custom columns (comma-separated). See `--list-columns`.
    #[arg(
        short = 'o',
        long = "format",
        value_name = "COLS",
        value_hint = ValueHint::Other,
        long_help = "Define custom columns (comma-separated).\n\nExample: -o pid,user,pcpu,etime,comm\n\nRun `ps --list-columns` to see all available column names.",
        help_heading = "Output format"
    )]
    pub format: Option<String>,

    /// Filter by PID (repeatable, comma-separated)
    #[arg(short = 'p', long = "pid", value_delimiter = ',', help_heading = "Filters")]
    pub pid: Vec<u32>,

    /// Filter by effective user name or SID, or enable user format when no argument
    #[arg(short = 'u', long = "user", value_delimiter = ',', help_heading = "Filters", num_args = 0..)]
    pub user: Vec<String>,

    /// User-oriented format (activated when -u is used without argument)
    #[arg(skip)]
    pub user_format: bool,

    /// Filter by real user name or SID
    #[arg(short = 'U', long = "User", value_delimiter = ',', help_heading = "Filters")]
    pub ruser: Vec<String>,

    /// Filter by command name (substring match)
    #[arg(short = 'C', long = "comm", value_delimiter = ',', help_heading = "Filters")]
    pub comm: Vec<String>,

    /// Sort by column (prefix with `-` to reverse), e.g. `--sort=-pcpu,pid`
    #[arg(
        long = "sort",
        value_delimiter = ',',
        long_help = "Sort by column. Prefix with `-` to reverse.\n\nExample: --sort=-pcpu,pid  (highest CPU first, then PID ascending)",
        help_heading = "Filters"
    )]
    pub sort: Vec<String>,

    /// Show process tree
    #[arg(long = "forest", help_heading = "Tree")]
    pub forest: bool,

    /// Omit column headers
    #[arg(long = "no-headers", help_heading = "Tree")]
    pub no_headers: bool,

    /// Use ASCII characters for tree drawing
    #[arg(long = "ascii", help_heading = "Tree")]
    pub ascii: bool,

    /// Skip command line lookup (faster on systems with many privileged processes)
    #[arg(
        long = "no-cmdline",
        long_help = "Skip the PEB-based command line lookup. Speeds up snapshotting on systems with many privileged processes whose PEB cannot be read. Columns that depend on cmdline (e.g. `args`, `cmd`) will show '?'.",
        help_heading = "Performance"
    )]
    pub no_cmdline: bool,

    /// Write output to FILE instead of stdout
    #[arg(long = "write", value_name = "FILE", value_hint = ValueHint::FilePath, help_heading = "Output")]
    pub write: Option<PathBuf>,

    /// Repeat header every N rows (0 = never)
    #[arg(long = "headers-repeat", default_value_t = 0, value_name = "N", help_heading = "Output")]
    pub headers_repeat: usize,

    /// Show version information
    #[arg(short = 'V', long = "version", action = ArgAction::Version)]
    pub version: (),

    /// Print all available column names and exit
    #[arg(long = "list-columns", help_heading = "Output")]
    pub list_columns: bool,
}

impl Cli {
    pub fn default_columns() -> Vec<&'static str> {
        vec!["pid", "tty", "time", "comm"]
    }

    pub fn columns_to_render(&self) -> Vec<String> {
        if let Some(spec) = &self.format {
            return spec
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        if self.full {
            return vec!["uid", "pid", "ppid", "c", "stime", "tty", "time", "cmd"]
                .into_iter()
                .map(String::from)
                .collect();
        }
        if self.extra_full {
            return vec![
                "uid", "pid", "ppid", "c", "sz", "rss", "psr", "stime", "tty", "time",
                "comm", "args",
            ]
            .into_iter()
            .map(String::from)
            .collect();
        }
        if self.long {
            return vec!["f", "uid", "pid", "ppid", "pri", "ni", "vsz", "rss", "wchan", "stat", "tty", "time", "comm"]
                .into_iter()
                .map(String::from)
                .collect();
        }
        if self.jobs {
            return vec!["pid", "pgid", "sid", "tty", "time", "comm"]
                .into_iter()
                .map(String::from)
                .collect();
        }
        if self.user_format {
            return vec!["user", "pid", "pcpu", "pmem", "vsz", "rss", "tty", "stat", "start", "time", "command"]
                .into_iter()
                .map(String::from)
                .collect();
        }
        Self::default_columns()
            .into_iter()
            .map(String::from)
            .collect()
    }
}
