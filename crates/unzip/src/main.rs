mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::ZipArchive;

fn list_archive(cli: &Cli) -> io::Result<()> {
    let archive_path = cli
        .archive
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No archive specified"))?;
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let path = file.name();
        let size = file.size();
        let is_dir = file.is_dir();

        println!(
            "{} {} {}",
            if is_dir { "d" } else { "-" },
            format!("{:>12}", size),
            path
        );
    }

    Ok(())
}

fn extract_archive(cli: &Cli) -> Result<bool, ZipError> {
    let archive_path = cli.archive.as_ref().ok_or_else(|| {
        ZipError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No archive specified",
        ))
    })?;
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let dest_path = if let Some(dir) = &cli.directory {
        PathBuf::from(dir)
    } else {
        PathBuf::from(".")
    };

    if !dest_path.exists() {
        fs::create_dir_all(&dest_path)?;
    }

    let mut skipped_unsafe = false;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Reject "zip slip" entries: names that escape the destination
        // directory via `..` components, absolute paths, or drive/UNC
        // prefixes must never be joined onto the destination.
        let Some(enclosed) = file.enclosed_name() else {
            eprintln!("unzip: skipping entry with unsafe path: '{}'", file.name());
            skipped_unsafe = true;
            continue;
        };

        let file_path = dest_path.join(enclosed);

        if !cli.quiet {
            println!("{}", file_path.display());
        }

        if file.is_dir() {
            if !file_path.exists() {
                fs::create_dir_all(&file_path)?;
            }
        } else {
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            if file_path.exists() && !cli.overwrite {
                eprintln!(
                    "unzip: cannot overwrite existing file: {}",
                    file_path.display()
                );
                continue;
            }

            if file_path.exists() {
                // An overwrite must not fail on the Windows read-only
                // attribute; clear it first (cp-style) so File::create
                // can replace the content.
                clear_readonly_attribute(&file_path);
            }

            // Stream the entry to disk instead of buffering the whole
            // decompressed content in memory (avoids memory amplification
            // on large entries / zip bombs).
            let mut out_file = File::create(&file_path)?;
            io::copy(&mut file, &mut out_file)?;
            out_file.flush()?;
        }
    }

    Ok(skipped_unsafe)
}

/// Clear FILE_ATTRIBUTE_READONLY on `path` so an overwrite can replace it.
/// Uses the std permissions API, so no extra Windows dependency is needed.
#[cfg(windows)]
#[allow(clippy::permissions_set_readonly_false)] // the intended action here
fn clear_readonly_attribute(path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

#[cfg(not(windows))]
fn clear_readonly_attribute(_path: &Path) {}

fn main() {
    let cli = Cli::parse();

    if cli.list {
        if let Err(e) = list_archive(&cli) {
            eprintln!("unzip: error: {}", e);
            std::process::exit(1);
        }
    } else if cli.archive.is_some() {
        match extract_archive(&cli) {
            Ok(skipped_unsafe) => {
                if skipped_unsafe {
                    // At least one entry was refused; signal a partial
                    // failure so scripts can detect unextracted content.
                    std::process::exit(2);
                }
            }
            Err(e) => {
                eprintln!("unzip: error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("unzip: no archive specified");
        std::process::exit(1);
    }
}
