use clap::Parser;
use std::path::PathBuf;

/// Copy files and directories — a Windows port of the Linux cp command.
#[derive(Parser, Debug)]
#[command(
    name = "cp",
    version,
    about,
    override_usage = "cp [OPTIONS] <SOURCE>... [DESTINATION]"
)]
pub struct Args {
    /// Copy directories recursively
    #[arg(short = 'r', short_alias = 'R', long = "recursive")]
    pub recursive: bool,

    /// Like -pR: preserve the mappable attributes and recurse. The GNU -d
    /// (no-dereference) part has no Windows mapping and is not applied.
    #[arg(short = 'a', long = "archive")]
    pub archive: bool,

    /// Copy only when the SOURCE file is newer than the destination file
    /// or when the destination file is missing
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Copy all SOURCE arguments into DIRECTORY
    #[arg(short = 't', long = "target-directory", value_name = "DIRECTORY")]
    pub target_directory: Option<PathBuf>,

    /// Treat DESTINATION as a regular file, never look inside it as a directory
    #[arg(short = 'T', long = "no-target-directory")]
    pub no_target_directory: bool,

    /// Overwrite existing files without prompting
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Prompt before overwrite
    #[arg(short = 'i', long = "interactive", overrides_with = "no_clobber")]
    pub interactive: bool,

    /// Do not overwrite an existing file
    #[arg(short = 'n', long = "no-clobber", overrides_with = "interactive")]
    pub no_clobber: bool,

    /// Preserve file attributes
    #[arg(short = 'p', long = "preserve")]
    pub preserve: bool,

    /// Create symbolic links instead of copying
    #[arg(short = 's', long = "symbolic-link")]
    pub symbolic_link: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The source file(s) and, unless -t is given, the final DESTINATION as
    /// the last operand. Which operand is the destination follows GNU's
    /// position rules and is decided in main, not by clap.
    #[arg(value_name = "SOURCE", num_args = 1..)]
    pub sources: Vec<PathBuf>,
}
