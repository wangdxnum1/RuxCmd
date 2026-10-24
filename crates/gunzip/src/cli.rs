use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gunzip", version = "0.1.0", about = "Decompress gzip files")]
pub struct Cli {
    #[arg(short = 'k', long = "keep", help = "Keep the original compressed file")]
    pub keep: bool,

    #[arg(short = 'f', long = "force", help = "Force overwrite of the output file")]
    pub force: bool,

    #[arg(short = 'v', long = "verbose", help = "Show progress information")]
    pub verbose: bool,

    /// Write the decompressed data to stdout and keep the source archive.
    #[arg(short = 'c', long = "stdout", help = "Write output to stdout, keep source")]
    pub stdout: bool,

    /// List compression statistics (compressed size, uncompressed size,
    /// ratio) instead of decompressing.
    #[arg(short = 'l', long = "list", help = "List compression statistics")]
    pub list: bool,

    #[arg(help = "The .gz file to decompress")]
    pub file: String,
}
