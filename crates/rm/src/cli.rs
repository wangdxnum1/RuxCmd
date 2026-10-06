use clap::Parser;
use std::path::PathBuf;

/// Remove files or directories — a Windows port of the Linux rm command.
#[derive(Parser, Debug)]
#[command(name = "rm", version, about)]
pub struct Args {
    /// Ignore nonexistent files and missing operands, never prompt
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Prompt before every removal
    #[arg(short = 'i', overrides_with_all = ["force"])]
    pub interactive: bool,

    /// Prompt once before removing more than three files, or when removing recursively;
    /// less intrusive than -i, while still giving protection against most mistakes
    #[arg(short = 'I', overrides_with_all = ["force", "interactive"])]
    pub interactive_once: bool,

    /// Remove directories and their contents recursively
    #[arg(short = 'r', short_alias = 'R')]
    pub recursive: bool,

    /// Remove empty directories
    #[arg(short = 'd', long = "dir")]
    pub remove_empty_dirs: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// When removing recursively, skip files that reside on different filesystems
    #[arg(long = "one-file-system", requires = "recursive")]
    pub one_file_system: bool,

    /// Do not treat '/' specially (on Windows: do not protect C:\)
    #[arg(long = "no-preserve-root", overrides_with = "preserve_root")]
    pub no_preserve_root: bool,

    /// Do not remove '/' (on Windows: do not remove C:\) (default)
    #[arg(long = "preserve-root")]
    pub preserve_root: bool,

    /// The file(s) to remove
    #[arg(required_unless_present = "force", value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
