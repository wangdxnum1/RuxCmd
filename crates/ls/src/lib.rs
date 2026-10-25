//! Library entry point for the `ls` crate.
//!
//! Shared by the `ls` binary and the `vdir` binary (GNU vdir is ls with a
//! long listing format by default).

pub mod cli;
pub mod color;
pub mod display;
pub mod entry;
pub mod perm;
pub mod sort;

use clap::Parser as _;
use std::ffi::OsString;

/// Run the ls engine over `args` and return the process exit code.
///
/// `args` mirrors `argv`: the first item is the program name (skipped, as
/// clap does for `std::env::args_os()`), the rest are the user-facing
/// arguments. The `vdir` binary calls this with a `"vdir"` placeholder
/// followed by `-l` and the user's own arguments.
pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let args = cli::Args::parse_from(args);
    let paths = if args.files.is_empty() {
        vec![std::path::PathBuf::from(".")]
    } else {
        args.files.clone()
    };

    let mut exit_code = 0i32;
    let show_total = paths.len() > 1 && args.long;
    let has_multiple_sources = paths.len() > 1;

    for (idx, path) in paths.iter().enumerate() {
        match entry::collect_entries(path, &args) {
            Ok(entries) => {
                if has_multiple_sources {
                    if idx > 0 {
                        println!();
                    }
                    println!("{}:", path.display());
                }
                if show_total {
                    let total_blocks: u64 = entries
                        .iter()
                        .filter(|e| !e.name.starts_with('.') || args.all)
                        .map(|e| e.block_count() * 512 / 1024)
                        .sum();
                    println!("total {}", total_blocks);
                }
                display::print_entries(&entries, &args);
            }
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", path.display(), e);
                exit_code = 2;
            }
        }
    }

    exit_code
}
