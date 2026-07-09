use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "uniq", about = "Report or omit repeated lines", version = "0.1.0", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'c', long = "count", help = "Prefix lines by the number of occurrences")]
    pub count: bool,

    #[arg(short = 'd', long = "repeated", help = "Only print duplicate lines")]
    pub duplicate_only: bool,

    #[arg(short = 'u', long = "unique", help = "Only print unique lines")]
    pub unique_only: bool,

    #[arg(short = 'i', long = "ignore-case", help = "Ignore case when comparing")]
    pub ignore_case: bool,

    #[arg(short = 'f', long = "skip-fields", help = "Avoid comparing the first N fields")]
    pub skip_fields: Option<usize>,

    #[arg(short = 'v', long = "version", help = "Print version information")]
    pub version: bool,

    #[arg(help = "Input file (default: stdin)")]
    pub file: Option<String>,
}