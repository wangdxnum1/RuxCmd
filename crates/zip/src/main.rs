mod cli;

use clap::Parser;
use cli::Args;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("zip: {}", e);
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), String> {
    let file = File::create(&args.zipfile)
        .map_err(|e| format!("cannot create '{}': {}", args.zipfile.display(), e))?;
    let mut writer = ZipWriter::new(file);

    for path in &args.files {
        add_path(&mut writer, path, args)?;
    }

    writer
        .finish()
        .map_err(|e| format!("failed to finish archive: {e}"))?;
    Ok(())
}

fn add_path<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    path: &Path,
    args: &Args,
) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("{}: {}", path.display(), e))?;

    if metadata.is_dir() {
        add_directory(writer, path, args)?;
        if !args.recurse {
            return Ok(());
        }
        let mut children: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|e| format!("cannot read directory '{}': {}", path.display(), e))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        children.sort();
        for child in children {
            add_path(writer, &child, args)?;
        }
        return Ok(());
    }

    add_file(writer, path, args)
}

fn add_directory<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    path: &Path,
    args: &Args,
) -> Result<(), String> {
    let name = archive_name(path, true);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);
    writer
        .add_directory(name.as_str(), options)
        .map_err(|e| format!("cannot add '{}': {}", path.display(), e))?;
    if !args.quiet {
        println!("  adding: {name}");
    }
    Ok(())
}

fn add_file<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    path: &Path,
    args: &Args,
) -> Result<(), String> {
    let name = archive_name(path, false);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    writer
        .start_file(name.as_str(), options)
        .map_err(|e| format!("cannot add '{}': {}", path.display(), e))?;
    let mut source = File::open(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    io::copy(&mut source, writer).map_err(|e| format!("cannot add '{}': {}", path.display(), e))?;
    if !args.quiet {
        println!("  adding: {name}");
    }
    Ok(())
}

/// Archive names use forward slashes and keep the path as the user typed it,
/// minus a leading `./`; directories get a trailing slash.
fn archive_name(path: &Path, is_dir: bool) -> String {
    let mut name = path.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = name.strip_prefix("./") {
        name = stripped.to_string();
    }
    if is_dir && !name.ends_with('/') {
        name.push('/');
    }
    name
}

#[cfg(test)]
mod tests {
    use super::archive_name;
    use std::path::Path;

    #[test]
    fn archive_names_use_forward_slashes() {
        assert_eq!(archive_name(Path::new("a\\b.txt"), false), "a/b.txt");
        assert_eq!(archive_name(Path::new("./a.txt"), false), "a.txt");
    }

    #[test]
    fn directories_get_a_trailing_slash() {
        assert_eq!(archive_name(Path::new("dir"), true), "dir/");
        assert_eq!(archive_name(Path::new("dir/"), true), "dir/");
    }
}
