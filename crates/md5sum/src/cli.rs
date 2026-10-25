use clap::Parser;
use std::path::PathBuf;

/// Compute MD5 hash sums of files — a Windows port of the Linux md5sum command.
#[derive(Parser, Debug)]
#[command(name = "md5sum", about, disable_version_flag = true)]
pub struct Args {
    /// End output records with NUL and do not escape file names
    #[arg(short = 'z', long = "zero")]
    pub zero: bool,
    /// Print version information and exit
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Read MD5 sums from the FILEs and check them
    #[arg(short = 'c', long = "check")]
    pub check: bool,

    /// Don't print OK for each successfully verified file
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Don't output anything; the exit status indicates success or failure
    #[arg(long = "status")]
    pub status: bool,

    /// Create a BSD-style checksum ("MD5 (file) = hash")
    #[arg(long = "tag")]
    pub tag: bool,

    /// The file(s) to compute MD5 hash for
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
