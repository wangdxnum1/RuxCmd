mod cli;

use clap::Parser;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.directories.is_empty() {
        eprintln!("rmdir: missing operand");
        std::process::exit(1);
    }

    let mut ok = true;

    for dir in &args.directories {
        // GNU prints the verbose diagnostic before the attempt, so a
        // directory whose removal fails still gets its line.
        if args.verbose {
            println!("rmdir: removing directory, '{}'", dir.display());
        }

        match fs::remove_dir(dir) {
            Ok(()) => {
                if args.parents && !remove_parents(dir, &args) {
                    ok = false;
                }
            }
            Err(e) => {
                if !ignorable_failure(&e, dir, args.ignore_fail_on_non_empty) {
                    eprintln!("rmdir: failed to remove '{}': {}", dir.display(), e);
                    ok = false;
                }
            }
        }
    }

    if !ok {
        std::process::exit(1);
    }
}

/// Remove the ancestors of a successfully removed directory, innermost
/// first. The climb stops at the first failure (reported unless
/// --ignore-fail-on-non-empty and the ancestor is merely non-empty) and at
/// the filesystem root, mirroring GNU's `remove_parents`.
fn remove_parents(dir: &Path, args: &cli::Args) -> bool {
    let mut current = dir.to_path_buf();

    loop {
        let parent = match current.parent() {
            // A relative path such as "a/b" ends in an empty parent; like
            // GNU, the climb simply stops there instead of touching ".".
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => break,
        };

        if args.verbose {
            println!("rmdir: removing directory, '{}'", parent.display());
        }

        match fs::remove_dir(&parent) {
            Ok(()) => current = parent,
            Err(e) => {
                if !ignorable_failure(&e, &parent, args.ignore_fail_on_non_empty) {
                    eprintln!(
                        "rmdir: failed to remove directory '{}': {}",
                        parent.display(),
                        e
                    );
                    return false;
                }
                // Stop quietly on an ignorable non-empty ancestor.
                return true;
            }
        }
    }

    true
}

/// --ignore-fail-on-non-empty only hides failures that are solely because
/// the directory is non-empty (GNU `ignorable_failure`).
fn ignorable_failure(e: &io::Error, dir: &Path, ignore: bool) -> bool {
    if !ignore {
        return false;
    }

    if is_non_empty_error(e) {
        return true;
    }

    // Windows may report permission-style errors for a non-empty directory;
    // verify the directory is actually non-empty before ignoring.
    if e.kind() == ErrorKind::PermissionDenied && directory_is_non_empty(dir) {
        return true;
    }

    false
}

/// Windows maps ERROR_DIR_NOT_EMPTY (145) to ErrorKind::DirectoryNotEmpty;
/// the raw code is also matched in case std has not classified it.
fn is_non_empty_error(e: &io::Error) -> bool {
    e.kind() == ErrorKind::DirectoryNotEmpty || e.raw_os_error() == Some(145)
}

fn directory_is_non_empty(dir: &Path) -> bool {
    fs::read_dir(dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}
