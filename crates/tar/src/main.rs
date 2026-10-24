mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
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
        // -C DIR uses chdir semantics: operands are resolved relative to
        // DIR (absolute operands are unaffected), and the names stored in
        // the archive are the operands as typed. Implemented by joining
        // the directory onto the disk path only.
        let disk_path: PathBuf = match &cli.directory {
            Some(dir) => Path::new(dir).join(file_path),
            None => PathBuf::from(file_path),
        };
        if !disk_path.exists() {
            eprintln!("tar: {}: No such file or directory", file_path);
            continue;
        }

        if cli.verbose {
            println!("{}", file_path);
        }

        append_entry(&mut tar, Path::new(file_path), &disk_path, &cli.exclude)?;
    }

    Ok(())
}

/// Appends one filesystem entry to the archive under `name`, recursing
/// into directories. `excludes` are matched against the archive-relative
/// name so `--exclude` prunes files and whole directory subtrees.
fn append_entry<W: io::Write>(
    tar: &mut tar::Builder<W>,
    name: &Path,
    disk_path: &Path,
    excludes: &[String],
) -> io::Result<()> {
    if is_excluded(&name.to_string_lossy(), excludes) {
        return Ok(());
    }

    let metadata = std::fs::metadata(disk_path)?;
    if metadata.is_dir() {
        tar.append_dir(name, disk_path)?;
        for entry in std::fs::read_dir(disk_path)? {
            let entry = entry?;
            let child_name = name.join(entry.file_name());
            append_entry(tar, &child_name, &entry.path(), excludes)?;
        }
    } else {
        let file = File::open(disk_path)?;
        let mut header = tar::Header::new_gnu();
        header.set_path(name)?;
        header.set_size(metadata.len());
        header.set_mode(0o644);
        let mtime = metadata
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

    Ok(())
}

/// Returns false when an entry path must be rejected on extraction:
/// absolute paths, Windows drive/UNC/verbatim prefixes, or any `..`
/// component would let the entry escape the extraction directory.
fn is_safe_entry_path(path: &Path) -> bool {
    if path.as_os_str().is_empty() {
        return false;
    }
    path.components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

/// True when `name` matches any exclude pattern. Patterns are simple globs
/// (`*` matches any run of characters, `?` one character). A pattern
/// without `/` matches any single path component (GNU's default
/// no-anchored matching for exclusion); a pattern with `/` is matched
/// against the whole path.
fn is_excluded(name: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    // Archive names always use `/`; normalize Windows separators from
    // operand paths the same way.
    let normalized = name.replace('\\', "/");
    patterns.iter().any(|pattern| {
        if pattern.contains('/') {
            glob_match(pattern.as_bytes(), normalized.as_bytes())
        } else {
            normalized
                .split('/')
                .any(|component| glob_match(pattern.as_bytes(), component.as_bytes()))
        }
    })
}

/// Iterative glob matcher supporting `*` and `?` with backtracking.
fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star, mut mark) = (usize::MAX, 0usize);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = p;
            mark = t;
            p += 1;
        } else if star != usize::MAX {
            p = star + 1;
            mark += 1;
            t = mark;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

/// GNU member selection: an empty member list selects everything;
/// otherwise a member matches by exact name (after stripping a leading
/// `./`) or as a directory prefix (`dir` selects `dir/...`).
fn member_matches(entry: &str, members: &[String]) -> bool {
    if members.is_empty() {
        return true;
    }
    let entry = entry.strip_prefix("./").unwrap_or(entry);
    members.iter().any(|member| {
        let member = member.strip_prefix("./").unwrap_or(member);
        let member = member.trim_end_matches('/');
        if member.is_empty() {
            return false;
        }
        entry == member
            || entry
                .strip_prefix(member)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// Opens the archive and wraps it in a gzip decoder when needed.
fn open_archive_reader(cli: &Cli) -> io::Result<BufReader<Box<dyn io::Read>>> {
    let file = File::open(&cli.file)?;
    let reader: Box<dyn io::Read> = if cli.gzip {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    Ok(BufReader::new(reader))
}

fn extract_archive(cli: &Cli) -> io::Result<()> {
    let mut archive = tar::Archive::new(open_archive_reader(cli)?);
    let base: &Path = cli
        .directory
        .as_deref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new(""));
    let mut rejected = false;
    let stdout = io::stdout();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        if !is_safe_entry_path(&path) {
            eprintln!(
                "tar: skipping '{}': entry path is unsafe (absolute paths, drive prefixes, and '..' components are not allowed)",
                path.display()
            );
            rejected = true;
            continue;
        }

        let name = path.to_string_lossy();
        if is_excluded(name.as_ref(), &cli.exclude) || !member_matches(&name, &cli.files) {
            continue;
        }

        if cli.to_stdout {
            if cli.verbose {
                // GNU writes member names to stderr when contents go to
                // stdout, keeping the data stream clean.
                eprintln!("{}", path.display());
            }
            if !entry.header().entry_type().is_dir() {
                let mut out = stdout.lock();
                io::copy(&mut entry, &mut out)?;
            }
        } else {
            if cli.verbose {
                println!("{}", path.display());
                io::stdout().flush()?;
            }

            // The entry path was validated above, so joining it onto the
            // -C directory (or the empty path) stays inside the target.
            let dest: PathBuf = base.join(&path);
            // Archives do not always carry parent directory entries, so
            // create any missing leading directories before unpacking.
            if let Some(parent) = dest.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            entry.unpack(&dest)?;
        }
    }

    stdout.lock().flush()?;

    if rejected {
        io::stdout().flush()?;
        std::process::exit(2);
    }

    Ok(())
}

fn list_archive(cli: &Cli) -> io::Result<()> {
    let mut archive = tar::Archive::new(open_archive_reader(cli)?);

    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_path_buf();

        let name = path.to_string_lossy();
        if is_excluded(name.as_ref(), &cli.exclude) || !member_matches(&name, &cli.files) {
            continue;
        }

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
