mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("zip: {}", e);
        std::process::exit(1);
    }
}

/// One pending entry to write: the archive name and where its content comes
/// from, plus whether the archive already held that name.
struct PendingEntry {
    name: String,
    source: EntrySource,
    /// GNU prints `updating:` when the archive already contained the name.
    updating: bool,
    /// False when `-u` decided the archived copy is already current.
    write: bool,
}

enum EntrySource {
    /// A directory marker (no payload).
    Directory,
    /// A file on disk; its mtime feeds `-u` comparisons.
    File { path: PathBuf },
}

fn run(args: &Args) -> Result<(), String> {
    // Collect the new entries first: names decide which existing entries the
    // update replaces, and -u compares modification times.
    let mut pending: Vec<PendingEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for path in &args.files {
        collect_path(path, args, &mut pending, &mut seen)?;
    }
    // Only entries actually written count as replaced for the carry-over.
    let replaced: HashSet<&str> = pending
        .iter()
        .filter(|e| e.updating && e.write)
        .map(|e| e.name.as_str())
        .collect();

    // GNU zip updates an existing archive in place: entries not named by this
    // invocation are carried over, replaced names are written as `updating:`.
    let archive_existed = args.zipfile.exists();
    let tmp = args.zipfile.with_extension("zip-ruxcmd-tmp");

    let file = File::create(&tmp).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot create '{}': {}", args.zipfile.display(), e)
    })?;
    let mut writer = ZipWriter::new(file);

    if archive_existed {
        carry_over(&mut writer, &args.zipfile, &replaced)?;
    }
    for entry in &pending {
        if !entry.write {
            continue;
        }
        match &entry.source {
            EntrySource::Directory => add_directory(&mut writer, entry, args)?,
            EntrySource::File { path } => add_file(&mut writer, entry, path, args)?,
        }
    }
    writer
        .finish()
        .map_err(|e| format!("failed to finish archive: {e}"))?;

    fs::rename(&tmp, &args.zipfile).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot update '{}': {}", args.zipfile.display(), e)
    })?;

    print_report(&args.zipfile, &pending, args)?;
    Ok(())
}

/// Walks one path argument into pending entries (directories recurse with -r).
fn collect_path(
    path: &Path,
    args: &Args,
    pending: &mut Vec<PendingEntry>,
    seen: &mut HashSet<String>,
) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("{}: {}", path.display(), e))?;

    if metadata.is_dir() {
        let name = archive_name(path, args.junk_paths, true);
        if seen.insert(name.clone()) {
            pending.push(PendingEntry {
                name,
                source: EntrySource::Directory,
                updating: false,
                write: true,
            });
        }
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
            collect_path(&child, args, pending, seen)?;
        }
        return Ok(());
    }

    let name = archive_name(path, args.junk_paths, false);
    let updating = args.zipfile.exists() && archive_contains(&args.zipfile, &name)?;
    // -u: the archived copy wins when it is at least as new as the source.
    let write = !(args.update && updating && {
        let archived = archived_mtime(&args.zipfile, &name).unwrap_or(std::time::UNIX_EPOCH);
        fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|modified| modified <= archived)
            .unwrap_or(false)
    });
    if seen.insert(name.clone()) {
        pending.push(PendingEntry {
            name,
            source: EntrySource::File {
                path: path.to_path_buf(),
            },
            updating,
            write,
        });
    }
    Ok(())
}

/// Carries every archive entry whose name is not being replaced into the new
/// archive, keeping its method, permissions and timestamps.  Carried entries
/// print nothing: GNU only reports the entries this invocation touched.
fn carry_over<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    archive_path: &Path,
    replaced: &HashSet<&str>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("cannot open '{}': {}", archive_path.display(), e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("cannot read '{}': {}", archive_path.display(), e))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("cannot read '{}': {}", archive_path.display(), e))?;
        let Some(name) = entry
            .enclosed_name()
            .map(|p| p.to_string_lossy().into_owned())
        else {
            continue;
        };
        // Replaced names are written fresh from the pending list instead.
        if replaced.contains(name.as_str()) {
            continue;
        }
        let options = SimpleFileOptions::default()
            .compression_method(entry.compression())
            .unix_permissions(unix_mode(
                &entry,
                if name.ends_with('/') { 0o755 } else { 0o644 },
            ))
            .last_modified_time(entry.last_modified().unwrap_or_default());
        let mut payload = Vec::new();
        entry
            .read_to_end(&mut payload)
            .map_err(|e| format!("cannot carry '{}': {}", name, e))?;
        if name.ends_with('/') {
            writer
                .add_directory(name.as_str(), options)
                .map_err(|e| format!("cannot carry '{}': {}", name, e))?;
        } else {
            writer
                .start_file(name.as_str(), options)
                .map_err(|e| format!("cannot carry '{}': {}", name, e))?;
            writer
                .write_all(&payload)
                .map_err(|e| format!("cannot carry '{}': {}", name, e))?;
        }
    }
    Ok(())
}

fn add_directory<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    entry: &PendingEntry,
    _args: &Args,
) -> Result<(), String> {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);
    writer
        .add_directory(entry.name.as_str(), options)
        .map_err(|e| format!("cannot add '{}': {}", entry.name, e))?;
    Ok(())
}

fn add_file<W: io::Write + io::Seek>(
    writer: &mut ZipWriter<W>,
    entry: &PendingEntry,
    path: &Path,
    args: &Args,
) -> Result<(), String> {
    let mut options = SimpleFileOptions::default()
        .compression_method(if args.store {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        })
        .unix_permissions(0o644);
    if args.best {
        options = options.compression_level(Some(9));
    }
    let mut source = File::open(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let modified = source
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .map(datetime_from_system_time);
    if let Some(stamp) = modified {
        options = options.last_modified_time(stamp);
    }
    writer
        .start_file(entry.name.as_str(), options)
        .map_err(|e| format!("cannot add '{}': {}", path.display(), e))?;
    io::copy(&mut source, writer).map_err(|e| format!("cannot add '{}': {}", path.display(), e))?;
    Ok(())
}

/// Per-entry report: `adding:` for new entries, `updating:` for replaced
/// ones, with the GNU compression percentage in parentheses.
fn print_report(archive_path: &Path, pending: &[PendingEntry], args: &Args) -> Result<(), String> {
    if args.quiet {
        return Ok(());
    }
    let file = File::open(archive_path)
        .map_err(|e| format!("cannot open '{}': {}", archive_path.display(), e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("cannot read '{}': {}", archive_path.display(), e))?;
    for entry in pending.iter().filter(|e: &&PendingEntry| e.write) {
        let stats = archive
            .by_name(&entry.name)
            .ok()
            .map(|e| (e.size(), e.compressed_size(), e.compression()));
        let is_dir = matches!(entry.source, EntrySource::Directory);
        let note = match stats {
            Some((raw, packed, method))
                if raw > 0 && !is_dir && method != CompressionMethod::Stored =>
            {
                let percent = ((raw as f64 - packed as f64) / raw as f64 * 100.0).round() as i64;
                format!("(deflated {percent}%)")
            }
            _ => "(stored 0%)".to_string(),
        };
        let verb = if entry.updating { "updating" } else { "adding" };
        println!("  {verb}: {} {note}", entry.name);
    }
    Ok(())
}

/// True when the archive exists and already holds `name`.
fn archive_contains(archive_path: &Path, name: &str) -> Result<bool, String> {
    let file = match File::open(archive_path) {
        Ok(file) => file,
        // A nonexistent archive means nothing is contained yet.
        Err(_) => return Ok(false),
    };
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("cannot read '{}': {}", archive_path.display(), e))?;
    let found = archive.by_name(name).is_ok();
    Ok(found)
}

/// Modification time of an archived entry, for `-u` comparisons.
fn archived_mtime(archive_path: &Path, name: &str) -> Option<std::time::SystemTime> {
    let file = File::open(archive_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    let entry = archive.by_name(name).ok()?;
    let stamp = entry.last_modified()?;
    // ZipDateTime: DOS date/time fields, offset-free local wall clock.
    let (year, month, day, hour, minute, second) = (
        stamp.year(),
        stamp.month(),
        stamp.day(),
        stamp.hour(),
        stamp.minute(),
        stamp.second(),
    );
    // Convert through the DOS-era epoch math without pulling chrono in.
    let days = days_from_civil(year as i64, month as i64, day as i64);
    let seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(seconds.max(0) as u64))
}

/// Days from 1970-01-01 to the given civil date, from ruxcmd-core.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    ruxcmd_core::timefmt::days_from_civil(y, m, d)
}

fn datetime_from_system_time(when: std::time::SystemTime) -> DateTime {
    let local = ruxcmd_core::timefmt::local_time(when);
    let fallback = (1980, 1, 1, 0, 0, 0);
    let (y, mo, d, h, mi, s) = match local {
        Some(t) if t.year >= 1980 => (
            t.year,
            u8::try_from(t.month).unwrap_or(1),
            u8::try_from(t.day).unwrap_or(1),
            u8::try_from(t.hour).unwrap_or(0),
            u8::try_from(t.minute).unwrap_or(0),
            u8::try_from(t.second & !1).unwrap_or(0),
        ),
        _ => fallback,
    };
    DateTime::from_date_and_time(y, mo, d, h, mi, s.min(59))
        .unwrap_or_else(|_| DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap())
}

/// The Unix mode bits recorded on an archive entry, or `fallback` when the
/// host system did not store them.
fn unix_mode<R: std::io::Read + std::io::Seek>(
    entry: &zip::read::ZipFile<'_, R>,
    fallback: u32,
) -> u32 {
    entry.unix_mode().unwrap_or(fallback)
}

/// Archive names use forward slashes and keep the path as the user typed it,
/// minus a leading `./`; directories get a trailing slash; -j keeps the
/// final component only.
fn archive_name(path: &Path, junk_paths: bool, is_dir: bool) -> String {
    let mut name = path.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = name.strip_prefix("./") {
        name = stripped.to_string();
    }
    if junk_paths {
        name = name
            .rsplit('/')
            .next()
            .map(|component| component.to_string())
            .unwrap_or(name);
    }
    if is_dir && !name.ends_with('/') {
        name.push('/');
    }
    name
}

#[cfg(test)]
mod tests {
    use super::{archive_name, days_from_civil};
    use std::path::Path;

    #[test]
    fn archive_names_use_forward_slashes() {
        assert_eq!(archive_name(Path::new("a\\b.txt"), false, false), "a/b.txt");
        assert_eq!(archive_name(Path::new("./a.txt"), false, false), "a.txt");
    }

    #[test]
    fn directories_get_a_trailing_slash() {
        assert_eq!(archive_name(Path::new("dir"), false, true), "dir/");
        assert_eq!(archive_name(Path::new("dir/"), false, true), "dir/");
    }

    #[test]
    fn junk_paths_keep_the_file_name_only() {
        assert_eq!(archive_name(Path::new("a/b/c.txt"), true, false), "c.txt");
        assert_eq!(archive_name(Path::new("dir"), true, true), "dir/");
    }

    #[test]
    fn civil_day_math_matches_known_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1980, 1, 1), 3652);
        assert_eq!(days_from_civil(2026, 11, 7), 20764);
    }
}
