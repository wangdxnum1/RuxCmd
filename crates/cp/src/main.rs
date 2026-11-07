mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    let args = cli::Args::parse();

    if args.sources.is_empty() {
        eprintln!("cp: missing file operand");
        std::process::exit(1);
    }

    // GNU refuses to combine the two destination selectors.
    if args.target_directory.is_some() && args.no_target_directory {
        eprintln!("cp: cannot combine --target-directory (-t) and --no-target-directory (-T)");
        std::process::exit(1);
    }
    if args.link && args.symbolic_link {
        eprintln!("cp: cannot make both hard and symbolic links");
        usage_error();
    }
    if (args.backup_bare || args.backup.is_some()) && args.no_clobber {
        eprintln!("cp: options --backup and --no-clobber are mutually exclusive");
        usage_error();
    }
    // Without -b/--backup, the VERSION_CONTROL environment variable selects
    // the default backup control (coreutils); an invalid value fails with
    // the documented argument error.
    // Backup selection: --backup=CONTROL overrides; bare -b takes the mode
    // from the VERSION_CONTROL environment (default existing); with neither,
    // no backups happen.  An invalid VERSION_CONTROL fails like GNU.
    let default_suffix = args.suffix.clone().unwrap_or_else(|| "~".to_string());
    let env_mode = |spec: &str| -> Option<Option<BackupMode>> {
        match spec {
            "none" | "off" | "0" => Some(None),
            "numbered" | "t" | "1" => Some(Some(BackupMode::Numbered)),
            "existing" | "nil" | "2" => Some(Some(BackupMode::Existing)),
            "simple" | "never" | "3" => Some(Some(BackupMode::Simple)),
            _ => None,
        }
    };
    let backup = if args.backup.is_some() {
        match args.backup.as_deref() {
            Some("none") | Some("off") | Some("0") => Backup {
                mode: None,
                suffix: default_suffix,
            },
            Some("numbered") | Some("t") | Some("1") => Backup {
                mode: Some(BackupMode::Numbered),
                suffix: default_suffix,
            },
            Some("existing") | Some("nil") | Some("2") => Backup {
                mode: Some(BackupMode::Existing),
                suffix: default_suffix,
            },
            Some("simple") | Some("never") | Some("3") => Backup {
                mode: Some(BackupMode::Simple),
                suffix: default_suffix,
            },
            Some(other) => {
                eprintln!("cp: invalid argument '{other}' for '--backup'");
                usage_error();
            }
            None => unreachable!("require_equals keeps --backup valued"),
        }
    } else if args.backup_bare {
        let mode = match std::env::var("VERSION_CONTROL").ok().as_deref() {
            None => Some(BackupMode::Existing),
            Some(spec) => match env_mode(spec) {
                Some(mode) => mode,
                None => {
                    eprintln!("cp: invalid argument '{spec}' for '$VERSION_CONTROL'");
                    eprintln!("Valid arguments are:");
                    eprintln!("  - 'none', 'off'");
                    eprintln!("  - 'simple', 'never'");
                    eprintln!("  - 'existing', 'nil'");
                    eprintln!("  - 'numbered', 't'");
                    eprintln!("Try 'cp --help' for more information.");
                    std::process::exit(1);
                }
            },
        };
        Backup {
            mode,
            suffix: default_suffix,
        }
    } else {
        Backup {
            mode: None,
            suffix: default_suffix,
        }
    };

    // Which operand is the destination follows GNU's position rules: with -t
    // every operand is a source and the option value is the destination, with
    // -T there must be exactly two operands, otherwise the last operand is
    // the destination.
    let sources: &[PathBuf];
    let destination: &Path;
    if let Some(target) = &args.target_directory {
        match fs::metadata(target) {
            Err(_) => {
                eprintln!(
                    "cp: failed to access '{}': No such file or directory",
                    target.display()
                );
                std::process::exit(1);
            }
            Ok(meta) if !meta.is_dir() => {
                eprintln!("cp: target '{}' is not a directory", target.display());
                std::process::exit(1);
            }
            Ok(_) => {}
        }
        sources = &args.sources;
        destination = target.as_path();
    } else if args.no_target_directory {
        if args.sources.len() > 2 {
            eprintln!("cp: extra operand '{}'", args.sources[2].display());
            usage_error();
        }
        if args.sources.len() < 2 {
            eprintln!(
                "cp: missing destination file operand after '{}'",
                args.sources[0].display()
            );
            usage_error();
        }
        sources = &args.sources[..1];
        destination = args.sources[1].as_path();
    } else {
        if args.sources.len() < 2 {
            eprintln!(
                "cp: missing destination file operand after '{}'",
                args.sources[0].display()
            );
            usage_error();
        }
        let split = args.sources.len() - 1;
        // With multiple sources the destination must be an existing directory,
        // otherwise each source would silently overwrite the same target file.
        if split > 1 && !(args.sources[split].exists() && args.sources[split].is_dir()) {
            eprintln!(
                "cp: target '{}' is not a directory",
                args.sources[split].display()
            );
            std::process::exit(1);
        }
        sources = &args.sources[..split];
        destination = args.sources[split].as_path();
    }

    let mut exit_code = 0i32;
    let mut had_error = false;

    for source in sources {
        if let Err(e) = copy_file(source, destination, &args, &backup, &mut had_error) {
            eprintln!("cp: {}", e);
            exit_code = 1;
        }
    }

    if had_error {
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

/// Backup settings parsed once in main: `--backup=CONTROL` (plain -b is
/// GNU's "existing" mode) and the -S suffix, which defaults to `~`.
#[derive(Clone, Copy, PartialEq)]
enum BackupMode {
    Numbered,
    Existing,
    Simple,
}

struct Backup {
    mode: Option<BackupMode>,
    suffix: String,
}

fn usage_error() -> ! {
    eprintln!("Try 'cp --help' for more information.");
    std::process::exit(1);
}

/// -a is GNU's shorthand for -pR (plus -d, which has no Windows mapping);
/// the explicit flags simply OR into the same decisions.
fn wants_recursive(args: &cli::Args) -> bool {
    args.recursive || args.archive
}

fn wants_preserve(args: &cli::Args) -> bool {
    args.preserve || args.archive
}

/// How an existing destination is treated. GNU cp overwrites by default; -n
/// and -i share one interactive setting and the last one on the command line
/// wins, while -f is an independent switch that does not settle that race.
#[derive(Clone, Copy)]
enum Overwrite {
    Allow,
    Prompt,
    Skip,
}

fn overwrite_decision(args: &cli::Args) -> Overwrite {
    if args.no_clobber {
        Overwrite::Skip
    } else if args.interactive {
        Overwrite::Prompt
    } else {
        Overwrite::Allow
    }
}

/// GNU -u copies only when the source is strictly newer than the destination
/// or the destination is missing: equal mtimes count as "not newer" (verified
/// against GNU coreutils 8.32). Both sides are 100ns FILETIMEs on Windows, so
/// the comparison is exact. Runs before -n/-i/-f and before the read-only
/// check, and skips silently even under -v.
fn source_not_newer(source_modified: io::Result<SystemTime>, dest: &Path) -> bool {
    let Ok(source_time) = source_modified else {
        return false;
    };
    let Ok(dest_metadata) = fs::metadata(dest) else {
        return false;
    };
    let Ok(dest_time) = dest_metadata.modified() else {
        return false;
    };
    source_time <= dest_time
}

/// GNU cp can only replace an unwritable destination when -f is in effect;
/// without it the copy fails with "Permission denied". On Windows the
/// equivalent of "unwritable" is the read-only attribute.
fn ensure_replaceable(path: &Path, force: bool) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("cannot stat '{}': {}", path.display(), e))?;
    if !metadata.permissions().readonly() {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "cannot create regular file '{}': Permission denied",
            path.display()
        ));
    }
    let _ = ruxcmd_core::fsutil::clear_readonly(path);
    Ok(())
}

fn copy_file(
    source: &Path,
    dest: &Path,
    args: &cli::Args,
    backup: &Backup,
    had_error: &mut bool,
) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "cannot stat '{}': No such file or directory",
            source.display()
        ));
    }

    let source_metadata =
        fs::metadata(source).map_err(|e| format!("cannot stat '{}': {}", source.display(), e))?;

    if source_metadata.is_dir() {
        if !wants_recursive(args) {
            return Err(format!(
                "-r not specified; omitting directory '{}'",
                source.display()
            ));
        }
        return copy_directory(source, dest, args, backup, had_error);
    }

    // GNU -T: the destination is never entered, so an existing directory
    // cannot receive a plain file copy.
    if args.no_target_directory && dest.is_dir() {
        return Err(format!(
            "cannot overwrite directory '{}' with non-directory",
            dest.display()
        ));
    }

    let dest_path = if args.no_target_directory {
        dest.to_path_buf()
    } else if dest.exists() && dest.is_dir() {
        match source.file_name() {
            Some(name) => dest.join(name),
            None => {
                return Err(format!(
                    "cannot determine target name for '{}'",
                    source.display()
                ));
            }
        }
    } else {
        dest.to_path_buf()
    };

    if args.update && source_not_newer(source_metadata.modified(), &dest_path) {
        return Ok(());
    }

    if !prepare_replace(source, &dest_path, args, backup)? {
        return Ok(());
    }

    if args.link {
        fs::hard_link(source, &dest_path).map_err(|e| {
            format!(
                "cannot create hard link '{}' to '{}': {}",
                dest_path.display(),
                source.display(),
                e
            )
        })?;
    } else if args.symbolic_link {
        #[cfg(windows)]
        symlink_file(source, &dest_path).map_err(|e| {
            format!(
                "cannot create symbolic link '{}': {}",
                dest_path.display(),
                e
            )
        })?;
        #[cfg(not(windows))]
        fs::symlink_file(source, &dest_path).map_err(|e| {
            format!(
                "cannot create symbolic link '{}': {}",
                dest_path.display(),
                e
            )
        })?;
    } else {
        fs::copy(source, &dest_path).map_err(|e| {
            format!(
                "cannot copy '{}' to '{}': {}",
                source.display(),
                dest_path.display(),
                e
            )
        })?;
    }

    if args.verbose {
        println!("'{}' -> '{}'", source.display(), dest_path.display());
    }

    if wants_preserve(args) {
        preserve_metadata(source, &dest_path)?;
    }

    Ok(())
}

/// Runs the overwrite, backup and hard-link pre-replace rules shared by
/// top-level copies and in-tree entries.  Returns false when the caller
/// should skip this destination entirely.
fn prepare_replace(
    source: &Path,
    dest_path: &Path,
    args: &cli::Args,
    backup: &Backup,
) -> Result<bool, String> {
    if dest_path.exists() {
        match overwrite_decision(args) {
            Overwrite::Skip => return Ok(false),
            Overwrite::Prompt => {
                if !prompt_yes_no(&format!("cp: overwrite '{}'? ", dest_path.display())) {
                    return Ok(false);
                }
            }
            Overwrite::Allow => {}
        }
    }
    let mut still_there = dest_path.exists();
    if let (true, Some(mode)) = (still_there, backup.mode) {
        make_backup(dest_path, mode, &backup.suffix)?;
        still_there = false;
    }
    if still_there && args.link && !args.force {
        return Err(format!(
            "cannot create hard link '{}' to '{}': File exists",
            dest_path.display(),
            source.display()
        ));
    }
    if still_there {
        if args.link {
            // GNU -f replaces an existing destination with the hard link.
            fs::remove_file(dest_path)
                .map_err(|e| format!("cannot remove '{}': {}", dest_path.display(), e))?;
        } else {
            ensure_replaceable(dest_path, args.force)?;
        }
    }
    Ok(true)
}

/// Rename an existing destination to its backup name: `name` + suffix for
/// simple backups, or the next free `name.~N~` for numbered ones.  GNU
/// "existing" numbers only when a numbered backup is already present.
fn make_backup(dest: &Path, mode: BackupMode, suffix: &str) -> Result<(), String> {
    let parent = dest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let Some(name) = dest.file_name().and_then(|n| n.to_str()) else {
        return Err(format!("cannot back up '{}'", dest.display()));
    };
    let next_numbered = |start: usize| -> PathBuf {
        let mut n = start;
        loop {
            let candidate = parent.join(format!("{name}.~{n}~"));
            if !candidate.exists() {
                return candidate;
            }
            n += 1;
        }
    };
    let backup_path = match mode {
        BackupMode::Simple => parent.join(format!("{name}{suffix}")),
        BackupMode::Numbered => next_numbered(1),
        BackupMode::Existing => {
            if next_numbered(1) != parent.join(format!("{name}.~1~")) {
                next_numbered(1)
            } else {
                parent.join(format!("{name}{suffix}"))
            }
        }
    };
    fs::rename(dest, &backup_path).map_err(|e| {
        format!(
            "cannot create backup file '{}': {}",
            backup_path.display(),
            e
        )
    })
}

/// Copy one file entry from inside a directory tree, applying the same
/// update, overwrite and read-only rules as a top-level copy.
fn copy_entry_file(
    source: &Path,
    dest_path: &Path,
    args: &cli::Args,
    backup: &Backup,
) -> Result<(), String> {
    if args.update {
        let source_metadata = fs::metadata(source)
            .map_err(|e| format!("cannot stat '{}': {}", source.display(), e))?;
        if source_not_newer(source_metadata.modified(), dest_path) {
            return Ok(());
        }
    }

    if !prepare_replace(source, dest_path, args, backup)? {
        return Ok(());
    }

    if args.link {
        fs::hard_link(source, dest_path).map_err(|e| {
            format!(
                "cannot create hard link '{}' to '{}': {}",
                dest_path.display(),
                source.display(),
                e
            )
        })?;
    } else {
        fs::copy(source, dest_path).map_err(|e| {
            format!(
                "cannot copy '{}' to '{}': {}",
                source.display(),
                dest_path.display(),
                e
            )
        })?;
    }

    if args.verbose {
        println!("'{}' -> '{}'", source.display(), dest_path.display());
    }

    if wants_preserve(args) {
        preserve_metadata(source, dest_path)?;
    }

    Ok(())
}

/// Apply the mappable parts of GNU `cp -p`: the read-only attribute and the
/// access/modification timestamps. NTFS has no equivalent for the remaining
/// POSIX mode bits or for owner/group, which are reported as unmappable in
/// docs/compatibility.md.
fn preserve_metadata(source: &Path, dest: &Path) -> Result<(), String> {
    let source_meta =
        fs::metadata(source).map_err(|e| format!("cannot stat '{}': {}", source.display(), e))?;

    let mut permissions = fs::metadata(dest)
        .map_err(|e| format!("cannot stat '{}': {}", dest.display(), e))?
        .permissions();
    permissions.set_readonly(source_meta.permissions().readonly());
    fs::set_permissions(dest, permissions)
        .map_err(|e| format!("failed to preserve mode of '{}': {}", dest.display(), e))?;

    set_timestamps(dest, &source_meta)?;
    Ok(())
}

#[cfg(windows)]
fn set_timestamps(dest: &Path, source_meta: &fs::Metadata) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use std::time::UNIX_EPOCH;
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_WRITE_ATTRIBUTES, OPEN_EXISTING, SetFileTime,
    };

    fn to_filetime(time: SystemTime) -> FILETIME {
        // FILETIME counts 100ns ticks since 1601-01-01.
        let ticks = time
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() / 100)
            .unwrap_or(0)
            + 116_444_736_000_000_000;
        FILETIME {
            dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        }
    }

    let wide: Vec<u16> = dest.as_os_str().encode_wide().chain([0]).collect();
    // FILE_FLAG_BACKUP_SEMANTICS makes the same call work for directories.
    let handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            FILE_WRITE_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }
    .map_err(|e| format!("cannot open '{}': {}", dest.display(), e))?;

    let accessed = source_meta.accessed().ok().map(to_filetime);
    let modified = source_meta.modified().ok().map(to_filetime);
    let accessed_ptr = accessed.as_ref().map(|t| t as *const FILETIME);
    let modified_ptr = modified.as_ref().map(|t| t as *const FILETIME);
    let result = unsafe { SetFileTime(handle, None, accessed_ptr, modified_ptr) };
    unsafe {
        let _ = CloseHandle(handle);
    }
    result.map_err(|e| {
        format!(
            "failed to preserve timestamps of '{}': {}",
            dest.display(),
            e
        )
    })
}

#[cfg(not(windows))]
fn set_timestamps(dest: &Path, source_meta: &fs::Metadata) -> Result<(), String> {
    let mut times = fs::FileTimes::new();
    if let Ok(accessed) = source_meta.accessed() {
        times = times.set_accessed(accessed);
    }
    if let Ok(modified) = source_meta.modified() {
        times = times.set_modified(modified);
    }
    fs::OpenOptions::new()
        .write(true)
        .open(dest)
        .and_then(|file| file.set_times(times))
        .map_err(|e| {
            format!(
                "failed to preserve timestamps of '{}': {}",
                dest.display(),
                e
            )
        })
}

fn copy_directory(
    source: &Path,
    dest: &Path,
    args: &cli::Args,
    backup: &Backup,
    had_error: &mut bool,
) -> Result<(), String> {
    let dest_path = if args.no_target_directory {
        // GNU -T: use the destination as-is. An existing directory receives
        // the source's contents; a non-directory destination is rejected.
        if dest.exists() && !dest.is_dir() {
            return Err(format!(
                "cannot overwrite non-directory '{}' with directory '{}'",
                dest.display(),
                source.display()
            ));
        }
        dest.to_path_buf()
    } else if ends_in_dot_component(source) {
        // `cp -r src/. dest` (also bare `.`/`..`): copy the contents of
        // source into dest itself. Dest must already be an existing
        // directory; unlike GNU cp we do not create it on demand.
        if !dest.is_dir() {
            return Err(format!("target '{}' is not a directory", dest.display()));
        }
        dest.to_path_buf()
    } else if dest.exists() && dest.is_dir() {
        match source.file_name() {
            Some(name) => dest.join(name),
            None => {
                return Err(format!(
                    "cannot determine target name for '{}'",
                    source.display()
                ));
            }
        }
    } else {
        dest.to_path_buf()
    };

    if !dest_path.exists() {
        fs::create_dir_all(&dest_path)
            .map_err(|e| format!("cannot create directory '{}': {}", dest_path.display(), e))?;
    }

    let entries = fs::read_dir(source)
        .map_err(|e| format!("cannot read directory '{}': {}", source.display(), e))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("cannot read entry in '{}': {}", source.display(), e))?;
        let entry_path = entry.path();
        let dest_entry_path = dest_path.join(entry.file_name());

        let entry_metadata = fs::metadata(&entry_path)
            .map_err(|e| format!("cannot stat '{}': {}", entry_path.display(), e))?;

        if entry_metadata.is_dir() {
            // A failing entry must not abort the remaining tree, but it does
            // make the overall cp fail (GNU exits 1 after reporting it).
            if let Err(e) = copy_directory(&entry_path, &dest_entry_path, args, backup, had_error) {
                eprintln!("cp: {}", e);
                *had_error = true;
            }
        } else if let Err(e) = copy_entry_file(&entry_path, &dest_entry_path, args, backup) {
            eprintln!("cp: {}", e);
            *had_error = true;
        }
    }

    // -p also stamps the directory itself; do it after the children so copying
    // them does not bump the timestamps again.
    if wants_preserve(args) {
        preserve_metadata(source, &dest_path)?;
    }

    Ok(())
}

/// True when the path's final component is `.` or `..` (e.g. `.`, `./`,
/// `src/.`, `src/..`), so the path carries no usable trailing name.
fn ends_in_dot_component(path: &Path) -> bool {
    use std::path::Component;
    match path.components().next_back() {
        Some(Component::CurDir) | Some(Component::ParentDir) => return true,
        _ => {}
    }
    // `components()` normalizes away interior/trailing `.` segments, so also
    // inspect the raw text for a trailing dot component such as `src/.`.
    let text = path.as_os_str().to_string_lossy();
    let text = text.trim_end_matches(['/', '\\']);
    matches!(text.rsplit(['/', '\\']).next(), Some(".") | Some(".."))
}

fn prompt_yes_no(prompt: &str) -> bool {
    // GNU cp writes its prompts to stderr, keeping stdout clean for scripts.
    eprint!("{}", prompt);
    let _ = io::stderr().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}

#[cfg(test)]
mod tests {
    use super::ends_in_dot_component;
    use std::path::Path;

    #[test]
    fn detects_trailing_dot_components() {
        for p in [
            ".", "./", "./.", "..", "../", "src/.", "src/..", "src/../", "a\\b\\..",
        ] {
            assert!(
                ends_in_dot_component(Path::new(p)),
                "expected dot tail: {p}"
            );
        }
        for p in [
            "src",
            "src/",
            "src/./sub",
            "src/..\\sub",
            "/",
            "\\",
            "C:\\",
            "C:\\dir",
            "",
        ] {
            assert!(
                !ends_in_dot_component(Path::new(p)),
                "expected no dot tail: {p}"
            );
        }
    }
}
