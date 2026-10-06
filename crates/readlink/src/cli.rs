use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "readlink", version, about, long_about = None, disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version, help = "Display version information")]
    pub version: (),

    #[arg(
        short = 'f',
        long = "canonicalize",
        help = "Canonicalize by following every symlink in every component of the given name recursively"
    )]
    pub canonicalize: bool,

    #[arg(
        short = 'n',
        long = "no-newline",
        help = "Do not output the trailing newline"
    )]
    pub no_newline: bool,

    #[arg(name = "FILE", help = "Symbolic link to read")]
    pub file: String,
}
