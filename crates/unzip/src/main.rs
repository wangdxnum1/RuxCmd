mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
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

fn extract_archive(cli: &Cli) -> Result<(), ZipError> {
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

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = dest_path.join(file.name());

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

            let mut out_file = File::create(&file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            out_file.write_all(&buffer)?;
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if cli.list {
        if let Err(e) = list_archive(&cli) {
            eprintln!("unzip: error: {}", e);
            std::process::exit(1);
        }
    } else if let Some(_) = &cli.archive {
        if let Err(e) = extract_archive(&cli) {
            eprintln!("unzip: error: {}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("unzip: no archive specified");
        std::process::exit(1);
    }
}
