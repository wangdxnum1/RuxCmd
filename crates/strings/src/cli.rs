use clap::Parser;
use std::path::PathBuf;

/// Character encodings; GNU strings names are case-sensitive, so the
/// CLI carries the raw string and `main` maps it explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    SevenBit,
    EightBit,
    Utf16BE,
    Utf16LE,
    Utf32BE,
    Utf32LE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radix {
    Octal,
    Hexadecimal,
    Decimal,
}

#[derive(Parser, Debug)]
#[command(name = "strings", about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue, help = "Print version")]
    pub version: bool,

    #[arg(
        short = 'n',
        long = "bytes",
        default_value = "4",
        help = "Set the minimum string length (default 4)"
    )]
    pub min_length: usize,

    #[arg(short = 'a', long = "all", action = clap::ArgAction::SetTrue, help = "Scan the entire file, including all data segments")]
    pub all: bool,

    #[arg(
        short = 'e',
        long = "encoding",
        help = "Select character encoding (s=7-bit, S=8-bit, b=16-bit BE, l=16-bit LE, B=32-bit BE, L=32-bit LE)"
    )]
    pub encoding: Option<String>,

    #[arg(short = 'f', long = "print-file-name", action = clap::ArgAction::SetTrue, help = "Print the filename before each string")]
    pub print_file_name: bool,

    #[arg(
        short = 't',
        long = "radix",
        help = "Print the offset before each string (o=octal, x=hexadecimal, d=decimal)"
    )]
    pub radix: Option<String>,

    #[arg(value_name = "FILE", help = "Files to scan for strings")]
    pub files: Vec<PathBuf>,
}
