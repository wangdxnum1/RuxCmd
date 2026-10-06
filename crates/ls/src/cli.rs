use clap::Parser;
use std::path::PathBuf;

/// List directory contents — a Windows port of the Linux ls command.
#[derive(Parser, Debug)]
#[command(name = "ls", version, about)]
pub struct Args {
    // ── Selection / filtering ──────────────────────────────────────
    /// Do not ignore entries starting with .
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Do not list implied . and ..
    #[arg(short = 'A', long = "almost-all")]
    pub almost_all: bool,

    /// List directories themselves, not their contents
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Do not list implied entries ending with ~
    #[arg(short = 'B', long = "ignore-backups")]
    pub ignore_backups: bool,

    /// Recursively list subdirectories
    #[arg(short = 'R', long = "recursive")]
    pub recursive: bool,

    /// List one file per line (same as -l on single-column display)
    #[arg(short = '1')]
    pub one_per_line: bool,

    // ── Output format ──────────────────────────────────────────────
    /// Use a long listing format
    #[arg(short = 'l')]
    pub long: bool,

    /// Comma-separated output
    #[arg(short = 'm')]
    pub comma: bool,

    /// List entries by columns (default)
    #[arg(short = 'C', overrides_with = "one_per_line")]
    pub columns: bool,

    /// List by lines (across, -x)
    #[arg(short = 'x')]
    pub across: bool,

    // ── Sorting ────────────────────────────────────────────────────
    /// Reverse order while sorting
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// Sort by size, largest first
    #[arg(short = 'S')]
    pub sort_size: bool,

    /// Sort by time, newest first
    #[arg(short = 't')]
    pub sort_time: bool,

    /// Sort by extension
    #[arg(short = 'X')]
    pub sort_extension: bool,

    /// Do not sort directory entries
    #[arg(short = 'U')]
    pub unsorted: bool,

    /// Natural sort of version numbers within text
    #[arg(short = 'v')]
    pub version_sort: bool,

    // ── Time ───────────────────────────────────────────────────────
    /// With -lt: sort by, and show, access time
    #[arg(short = 'u')]
    pub access_time: bool,

    /// With -lt: sort by, and show, status change time (on Windows: creation time)
    #[arg(short = 'c')]
    pub status_time: bool,

    /// Time style: full-iso, long-iso, iso, locale, +FORMAT
    #[arg(long = "time-style")]
    pub time_style: Option<String>,

    // ── Display / metadata ─────────────────────────────────────────
    /// With -l, print the author of each file (no-op on Windows)
    #[arg(long = "author")]
    pub author: bool,

    /// Print the index number of each file
    #[arg(short = 'i', long = "inode")]
    pub inode: bool,

    /// Print the allocated size of each file, in blocks
    #[arg(short = 's')]
    pub size: bool,

    /// Like -l, but do not list owner
    #[arg(short = 'g')]
    pub no_owner: bool,

    /// Like -l, but do not list group
    #[arg(short = 'G', long = "no-group")]
    pub no_group: bool,

    /// Like -l, but list numeric user and group IDs
    #[arg(short = 'n', long = "numeric-uid-gid")]
    pub numeric_uid_gid: bool,

    /// Human-readable sizes (e.g., 1K, 234M, 2G)
    #[arg(long = "human-readable")]
    pub human_readable: bool,

    /// Use 1024-byte blocks for file system usage; used only with -s
    #[arg(short = 'k', long = "kibibytes")]
    pub kibibytes: bool,

    /// With -l, scale sizes by SIZE (e.g., '--block-size=M')
    #[arg(long = "block-size")]
    pub block_size: Option<String>,

    /// Append indicator (one of */=>@|) to entries
    #[arg(short = 'F', long = "classify")]
    pub classify: bool,

    /// Append / indicator to directories
    #[arg(short = 'p', long = "indicator-style")]
    pub indicator_slash: bool,

    /// Color the output WHEN: always, auto (default), never
    #[arg(long = "color", default_missing_value = "always", num_args = 0..=1, require_equals = false)]
    pub color: Option<String>,

    /// Width of output in columns (0 = no limit)
    #[arg(short = 'W', long = "width")]
    pub width: Option<usize>,

    /// Enclose entry names in double quotes
    #[arg(short = 'Q', long = "quote-name")]
    pub quote_name: bool,

    /// Group directories before files
    #[arg(long = "group-directories-first")]
    pub group_dirs_first: bool,

    /// Do not follow symbolic links; show link info
    pub files: Vec<PathBuf>,
}

impl Args {
    /// Parse CLI args; print help/version and exit on error.
    pub fn parse_or_exit() -> Self {
        Self::parse()
    }

    /// Whether color is enabled based on --color flag and terminal detection.
    pub fn color_enabled(&self) -> bool {
        match self.color.as_deref() {
            Some("always") => true,
            Some("never") => false,
            _ => {
                // default == "auto": only when stdout is a terminal
                use std::io::IsTerminal;
                std::io::stdout().is_terminal()
            }
        }
    }

    pub fn show_hidden(&self) -> bool {
        self.all || self.almost_all
    }

    /// Whether to show owner column in -l output
    pub fn show_owner(&self) -> bool {
        !self.no_owner
    }

    /// Whether to show group column in -l output
    pub fn show_group(&self) -> bool {
        !self.no_group
    }
}
