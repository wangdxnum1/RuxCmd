use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Encoding {
    #[clap(name = "s")]
    SevenBit,
    #[clap(name = "S")]
    EightBit,
    #[clap(name = "b")]
    Utf16BE,
    #[clap(name = "l")]
    Utf16LE,
    #[clap(name = "B")]
    Utf32BE,
    #[clap(name = "L")]
    Utf32LE,
}

#[derive(Parser, Debug)]
#[command(name = "strings", about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", help = "Print version")]
    pub version: bool,

    #[arg(short = 'n', long = "bytes", default_value = "4", help = "Set the minimum string length (default 4)")]
    pub min_length: usize,

    #[arg(short = 'a', long = "all", help = "Scan the entire file, including all data segments")]
    pub all: bool,

    #[arg(short = 'e', long = "encoding", help = "Select character encoding (s=7-bit, S=8-bit, b=16-bit BE, l=16-bit LE, B=32-bit BE, L=32-bit LE)")]
    pub encoding: Option<Encoding>,

    #[arg(value_name = "FILE", help = "Files to scan for strings")]
    pub files: Vec<PathBuf>,
}