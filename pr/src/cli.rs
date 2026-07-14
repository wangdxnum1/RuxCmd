use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pr", version, about = "Paginate or columnate FILE(s) for printing", disable_version_flag = true)]
pub struct Args {
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

    #[arg(short = 'n', long = "number-lines", value_name = "SEP[N]", default_value = None)]
    pub number_lines: Option<String>,

    #[arg(short = 't', long = "omit-header", action = clap::ArgAction::SetTrue)]
    pub omit_header: bool,

    #[arg(short = '2', long = "two-column", action = clap::ArgAction::SetTrue)]
    pub two_column: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
