mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time;

fn create_archive(cli: &Cli) -> io::Result<()> {
    let file = File::create(&cli.file)?;
    let writer: Box<dyn io::Write> = if cli.gzip {
        Box::new(flate2::write::GzEncoder::new(
            file,
            flate2::Compression::default(),
        ))
    } else {
        Box::new(file)
    };

    let mut tar = tar::Builder::new(BufWriter::new(writer));

    for file_path in &cli.files {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("tar: {}: No such file or directory", file_path);
            continue;
        }

        if cli.verbose {
            println!("{}", file_path);
        }

        if path.is_dir() {
            tar.append_dir_all(file_path, path)?;
        } else {
            let file = File::open(path)?;
            let mut header = tar::Header::new_gnu();
            header.set_path(path)?;
            header.set_size(file.metadata()?.len());
            header.set_mode(0o644);
            let mtime = file
                .metadata()?
                .modified()
                .ok()
                .and_then(|t| t.duration_since(time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            if let Some(mtime) = mtime {
                header.set_mtime(mtime);
            }
            header.set_cksum();
            tar.append(&header, file)?;
        }
    }

    Ok(())
}

fn extract_archive(cli: &Cli) -> io::Result<()> {
    let file = File::open(&cli.file)?;
    let reader: Box<dyn io::Read> = if cli.gzip {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let mut archive = tar::Archive::new(BufReader::new(reader));

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        if cli.verbose {
            println!("{}", path.display());
        }

        entry.unpack(&path)?;
    }

    Ok(())
}

fn list_archive(cli: &Cli) -> io::Result<()> {
    let file = File::open(&cli.file)?;
    let reader: Box<dyn io::Read> = if cli.gzip {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let mut archive = tar::Archive::new(BufReader::new(reader));

    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_path_buf();

        if cli.verbose {
            let header = entry.header();
            let size = header.size().unwrap_or(0);
            let file_type = if header.entry_type().is_dir() {
                "d"
            } else {
                "-"
            };
            let mode = header
                .mode()
                .map(|m| format!("{:04o}", m & 0o7777))
                .unwrap_or_else(|_| "????".to_string());
            let mtime_str = header
                .mtime()
                .map(|t| format!("{}", t))
                .unwrap_or_else(|_| "??????????".to_string());

            println!(
                "{} {} {:>12} {} {}",
                file_type,
                mode,
                size,
                mtime_str,
                path.display()
            );
        } else {
            println!("{}", path.display());
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.create {
        create_archive(&cli)
    } else if cli.extract {
        extract_archive(&cli)
    } else if cli.list {
        list_archive(&cli)
    } else {
        eprintln!("tar: Must specify one of -c, -x, or -t");
        std::process::exit(1);
    }
}
