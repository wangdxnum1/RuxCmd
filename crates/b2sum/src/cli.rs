use clap::Parser;

/// Compute or check BLAKE2b checksums - a Windows port of Linux b2sum.
#[derive(Parser, Debug)]
#[command(
    name = "b2sum",
    version,
    about,
    disable_version_flag = true,
    after_help = "With no FILE, or when FILE is -, read standard input."
)]
pub struct Args {
    /// Create a BLAKE2b checksum of the files
    #[arg(short = 'c', long = "check")]
    pub check: bool,

    /// Suppress OK lines when verifying
    #[arg(long = "quiet")]
    pub quiet: bool,

    /// Exit without printing anything when verifying
    #[arg(long = "status")]
    pub status: bool,

    /// Create a BSD-style checksum
    #[arg(long = "tag")]
    pub tag: bool,

    /// DIGEST bits; must be a multiple of 8 and no longer than 512
    #[arg(
        short = 'l',
        long = "length",
        value_name = "BITS",
        default_value_t = 512
    )]
    pub length: u32,

    /// Files to digest (default: standard input)
    #[arg(value_name = "FILE")]
    pub files: Vec<std::path::PathBuf>,
}

pub fn parse_args() -> Args {
    let args = Args::parse();
    if args.length == 0 || args.length > 512 || args.length % 8 != 0 {
        eprintln!(
            "b2sum: invalid length: '{}' is not a multiple of 8 between 8 and 512",
            args.length
        );
        std::process::exit(1);
    }
    args
}
