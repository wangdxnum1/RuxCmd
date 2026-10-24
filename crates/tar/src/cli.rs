use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "tar", version = "0.1.0", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'c', long = "create", conflicts_with_all = ["extract", "list"])]
    pub create: bool,

    #[arg(short = 'x', long = "extract", conflicts_with_all = ["create", "list"])]
    pub extract: bool,

    #[arg(short = 't', long = "list", conflicts_with_all = ["create", "extract"])]
    pub list: bool,

    #[arg(short = 'f', long = "file", required = true)]
    pub file: String,

    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[arg(short = 'z', long = "gzip")]
    pub gzip: bool,

    /// Create/extract relative to DIR (chdir semantics, implemented as
    /// path-prefix concatenation).
    #[arg(short = 'C', long = "directory", value_name = "DIR")]
    pub directory: Option<String>,

    /// Extract: write entry contents to stdout instead of disk.
    #[arg(short = 'O', long = "to-stdout")]
    pub to_stdout: bool,

    /// Glob (`*`, `?`) patterns; entries matching at any path component
    /// are skipped (create/extract/list).
    #[arg(long = "exclude", value_name = "PATTERN")]
    pub exclude: Vec<String>,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Create mode: files/directories to archive. Extract/list mode:
    /// optional member names to select (exact name or directory prefix).
    #[arg(name = "FILES", required_unless_present_any = ["list", "extract"])]
    pub files: Vec<String>,
}
