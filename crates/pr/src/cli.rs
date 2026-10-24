use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pr",
    version,
    about = "Paginate or columnate FILE(s) for printing",
    disable_version_flag = true,
    disable_help_flag = true
)]
pub struct Args {
    // -h is the page header in GNU pr; help is long-only.
    #[arg(long = "help", action = clap::ArgAction::Help)]
    pub help: (),

    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'd', long = "double-space", action = clap::ArgAction::SetTrue)]
    pub double_space: bool,

    #[arg(short = 'l', long = "length", default_value_t = 66usize)]
    pub length: usize,

    #[arg(short = 'o', long = "indent", default_value_t = 0usize)]
    pub indent: usize,

    #[arg(short = 'W', long = "page-width", default_value_t = 72usize)]
    pub page_width: usize,

    #[arg(short = 'h', long = "header", value_name = "HEADER")]
    pub header: Option<String>,

    #[arg(short = 'f', long = "form-feed", action = clap::ArgAction::SetTrue)]
    pub form_feed: bool,

    // GNU `-n[SEP[DIGITS]]`: the value is optional and, when present, must be
    // attached (`-n:5`, `-n5`). Bare `-n` means TAB separator + width 5; main
    // rewrites a bare `-n` token so clap does not swallow the next operand.
    #[arg(
        short = 'n',
        long = "number-lines",
        value_name = "SEP[N]",
        num_args = 0..=1,
        default_missing_value = ""
    )]
    pub number_lines: Option<String>,

    #[arg(short = 't', long = "omit-header", action = clap::ArgAction::SetTrue)]
    pub omit_header: bool,

    #[arg(short = '2', long = "two-column", action = clap::ArgAction::SetTrue)]
    pub two_column: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
