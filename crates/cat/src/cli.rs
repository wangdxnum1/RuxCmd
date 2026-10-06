use clap::Parser;
use std::path::PathBuf;

/// Concatenate files and print on the standard output — a Windows port of the Linux cat command.
#[derive(Parser, Debug)]
#[command(name = "cat", version, about)]
pub struct Args {
    /// Number all output lines
    #[arg(short = 'n', long = "number")]
    pub number: bool,

    /// Number non-empty output lines
    #[arg(short = 'b', long = "number-nonblank", overrides_with = "number")]
    pub number_nonblank: bool,

    /// Squeeze multiple consecutive blank lines into one
    #[arg(short = 's', long = "squeeze-blank")]
    pub squeeze_blank: bool,

    /// Display $ at end of each line
    #[arg(short = 'E', long = "show-ends")]
    pub show_ends: bool,

    /// Display TAB characters as ^I
    #[arg(short = 'T', long = "show-tabs")]
    pub show_tabs: bool,

    /// Display all non-printing characters except LFD and TAB
    #[arg(short = 'v', long = "show-nonprinting")]
    pub show_nonprinting: bool,

    /// Equivalent to -vET
    #[arg(short = 'A', long = "show-all", overrides_with_all = ["show_ends", "show_tabs", "show_nonprinting"])]
    pub show_all: bool,

    /// Use unbuffered I/O
    #[arg(short = 'u', long = "unbuffered")]
    pub unbuffered: bool,

    /// The file(s) to concatenate
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
