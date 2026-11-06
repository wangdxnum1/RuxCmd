mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, GetCompressedFileSizeW, GetFileInformationByHandle,
};

fn main() {
    let args = Args::parse();

    // GNU parses --max-depth while processing options, so an invalid value is
    // reported before any other conflict.
    let max_depth = args.max_depth.as_deref().map(|raw| {
        parse_max_depth(raw)
            .unwrap_or_else(|| usage_error(&format!("invalid maximum depth '{raw}'")))
    });

    if args.all && args.summarize {
        usage_error("cannot both summarize and show all entries");
    }
    if args.summarize
        && let Some(depth) = max_depth
    {
        if depth == 0 {
            eprintln!("du: warning: summarizing is the same as using --max-depth=0");
        } else {
            eprintln!("du: warning: summarizing conflicts with --max-depth={depth}");
            eprintln!("Try 'du --help' for more information.");
            std::process::exit(1);
        }
    }

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // GNU prints one line per subdirectory, children before their parent.
    // -s and --max-depth only restrict printing, never counting.
    let mut counter = Counter {
        apparent: args.apparent_size || args.bytes,
        human: args.human_readable,
        bytes: args.bytes,
        all: args.all,
        max_print: if args.summarize {
            0
        } else {
            max_depth.unwrap_or(u64::MAX)
        },
        one_file_system: args.one_file_system,
        excludes: args.exclude.clone(),
        root_volume: None,
        volume_of: Box::new(volume_key),
        seen: HashSet::new(),
        seen_dirs: HashSet::new(),
        had_error: false,
    };

    let mut grand_total = 0;
    for path in &paths {
        let base = operand_name(path);
        counter.root_volume = (counter.volume_of)(path);
        // GNU omits the line entirely for an argument that repeats an
        // already-seen entry or is excluded, so only print when a value
        // was returned.
        if let Some(size) = counter.total(path, &base, 0, &mut out) {
            grand_total += size;
            let _ = writeln!(out, "{}\t{}", counter.display(size), base);
        }
    }

    if args.total {
        let _ = writeln!(out, "{}\ttotal", counter.display(grand_total));
    }

    if counter.had_error {
        std::process::exit(1);
    }
}

/// GNU-style usage failure: message, Try line, exit 1.
fn usage_error(message: &str) -> ! {
    eprintln!("du: {message}");
    eprintln!("Try 'du --help' for more information.");
    std::process::exit(1);
}

/// Entries print with `/` separators like GNU; a trailing slash on the
/// operand is dropped so children join cleanly.
fn operand_name(path: &Path) -> String {
    let mut name = path.to_string_lossy().replace('\\', "/");
    while name.len() > 1 && name.ends_with('/') {
        name.pop();
    }
    name
}

/// Volume lookup used by `-x`; injectable so tests can fake file system
/// boundaries. Returns the volume key, or `None` when unknown.
type VolumeOf = Box<dyn Fn(&Path) -> Option<String>>;

struct Counter {
    /// Count logical sizes instead of allocated disk space.
    apparent: bool,
    human: bool,
    /// Print raw bytes instead of 1K blocks.
    bytes: bool,
    /// -a: also print a line for every file.
    all: bool,
    /// Only entries up to this depth (operands are depth 0) print; -s and
    /// --max-depth only restrict printing, never counting.
    max_print: u64,
    /// -x: never descend onto a different volume.
    one_file_system: bool,
    excludes: Vec<String>,
    /// Volume of the operand currently being processed (for -x).
    root_volume: Option<String>,
    volume_of: VolumeOf,
    seen: HashSet<(u32, u64, u64)>,
    /// Directories already visited; a repeated argument is skipped entirely.
    seen_dirs: HashSet<PathBuf>,
    had_error: bool,
}

impl Counter {
    /// Returns `None` when the entry was skipped: an excluded entry, an entry
    /// on a foreign volume (-x), an already counted entry, or an unreadable
    /// path (already reported on stderr). Nested directories print their own
    /// line when depth allows; the caller prints operands.
    fn total(&mut self, path: &Path, rel: &str, depth: u64, out: &mut dyn Write) -> Option<u64> {
        if is_excluded(rel, &self.excludes) {
            return None;
        }
        if depth > 0 && self.crosses_file_system(path) {
            return None;
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("du: {}: {}", path.display(), e);
                self.had_error = true;
                return None;
            }
        };

        if metadata.is_dir() {
            // GNU counts every inode once; a directory reached again (for
            // example `du a ./a`) produces no line at all.
            let identity = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            if !self.seen_dirs.insert(identity) {
                return None;
            }
            let mut size = 0;
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let child = format!("{}/{}", rel, entry.file_name().to_string_lossy());
                        size += self
                            .total(&entry.path(), &child, depth + 1, out)
                            .unwrap_or(0);
                    }
                }
                Err(e) => {
                    eprintln!("du: {}: {}", path.display(), e);
                    self.had_error = true;
                }
            }
            if depth > 0 && depth <= self.max_print {
                let _ = writeln!(out, "{}\t{}", self.display(size), rel);
            }
            return Some(size);
        }

        // GNU counts each hard-linked file once per invocation.
        if let Some(id) = file_id(path)
            && !self.seen.insert(id)
        {
            return None;
        }

        let logical = metadata.len();
        let size = if self.apparent {
            logical
        } else {
            allocated_size(path).unwrap_or(logical)
        };
        if self.all && depth > 0 && depth <= self.max_print {
            let _ = writeln!(out, "{}\t{}", self.display(size), rel);
        }
        Some(size)
    }

    /// True when `path` sits on a volume different from the operand's.
    /// Unknown volumes are never treated as a boundary.
    fn crosses_file_system(&self, path: &Path) -> bool {
        if !self.one_file_system {
            return false;
        }
        match (&self.root_volume, (self.volume_of)(path)) {
            (Some(root), Some(volume)) => *root != volume,
            _ => false,
        }
    }

    fn display(&self, size: u64) -> String {
        if self.human {
            // GNU humanizes whole 1K blocks for allocated sizes but raw bytes
            // for apparent sizes.
            if self.apparent {
                human_size(size)
            } else {
                human_size(size.div_ceil(1024) * 1024)
            }
        } else if self.bytes {
            size.to_string()
        } else {
            size.div_ceil(1024).to_string()
        }
    }
}

/// True when `name` matches any exclude pattern. Names and patterns use `/`
/// separators. A pattern without `/` matches any single path component
/// (GNU's default no-anchored matching for exclusion); a pattern with `/` is
/// matched against every suffix of the path that starts at a component
/// boundary, so `b/y` excludes `a/b/y` and `b/*` empties `a/b` without
/// hiding the directory itself.
fn is_excluded(name: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let normalized = name.replace('\\', "/");
    patterns.iter().any(|pattern| {
        let pattern = pattern.replace('\\', "/");
        if pattern.contains('/') {
            normalized
                .match_indices('/')
                .map(|(i, _)| i + 1)
                .chain([0])
                .any(|start| glob_match(pattern.as_bytes(), &normalized.as_bytes()[start..]))
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

/// GNU parses the depth with `xstrtoumax(..., base 0)`: leading whitespace
/// and `+` are allowed, `0x` and `0` prefixes select hex and octal, and any
/// trailing junk, minus sign or overflow makes the value invalid.
fn parse_max_depth(raw: &str) -> Option<u64> {
    let text = raw.trim_start();
    let text = text.strip_prefix('+').unwrap_or(text);
    let (radix, digits) =
        if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            (16, rest)
        } else if text.len() > 1 && text.starts_with('0') {
            (8, &text[1..])
        } else {
            (10, text)
        };
    if digits.is_empty() {
        return None;
    }
    u64::from_str_radix(digits, radix).ok()
}

/// Volume identity for `-x`: a drive (`c:`) or a UNC share
/// (`unc\server\share`), lowercased so raw and canonicalized forms of the
/// same path produce the same key.
fn volume_prefix(path: &str) -> Option<String> {
    if let Some(body) = path.strip_prefix(r"\\?\UNC\") {
        let (server, share) = first_two_components(body)?;
        return Some(format!("unc\\{server}\\{share}"));
    }
    if let Some(body) = path.strip_prefix(r"\\?\") {
        return drive_of(body);
    }
    if let Some(body) = path.strip_prefix(r"\\") {
        let (server, share) = first_two_components(body)?;
        return Some(format!("unc\\{server}\\{share}"));
    }
    drive_of(path)
}

fn drive_of(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        Some(path[..2].to_lowercase())
    } else {
        None
    }
}

fn first_two_components(rest: &str) -> Option<(String, String)> {
    let mut parts = rest.split('\\');
    let server = parts.next()?.to_lowercase();
    let share = parts.next()?.to_lowercase();
    (!server.is_empty() && !share.is_empty()).then_some((server, share))
}

/// The volume an entry lives on, or `None` when it cannot be determined.
fn volume_key(path: &Path) -> Option<String> {
    if let Ok(resolved) = fs::canonicalize(path)
        && let Some(key) = volume_prefix(&resolved.to_string_lossy())
    {
        return Some(key);
    }
    volume_prefix(&path.to_string_lossy())
}

/// Actual space on disk. Falls back to the logical size when the API is
/// unavailable; sparse files return their real (often much smaller) allocation.
fn allocated_size(path: &Path) -> Option<u64> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut high: u32 = 0;
    // SAFETY: `wide` is NUL-terminated and `high` is a valid out-pointer.
    let low =
        unsafe { GetCompressedFileSizeW(windows::core::PCWSTR(wide.as_ptr()), Some(&mut high)) };
    if low == u32::MAX && high == 0 {
        return None;
    }
    Some(((high as u64) << 32) | low as u64)
}

/// Volume serial plus file index identifies a file across hard links.
fn file_id(path: &Path) -> Option<(u32, u64, u64)> {
    let file = fs::File::open(path).ok()?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: the handle is owned by `file` and stays valid for the call.
    unsafe {
        GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut info).ok()?;
    }
    Some((
        info.dwVolumeSerialNumber,
        info.nFileIndexHigh as u64,
        info.nFileIndexLow as u64,
    ))
}

/// GNU `du -h` wording: raw numbers below 1K, one decimal below 10 units,
/// integers above that (`1.0K`, `4.9K`, `10K`, `1.0M`).
fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        return bytes.to_string();
    }
    let units = ["K", "M", "G", "T", "P"];
    let mut value = bytes as f64 / 1024.0;
    let mut idx = 0;
    while value >= 1024.0 && idx + 1 < units.len() {
        value /= 1024.0;
        idx += 1;
    }
    if value < 10.0 {
        format!("{:.1}{}", value, units[idx])
    } else {
        format!("{:.0}{}", value, units[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_sizes_match_gnu_formatting() {
        assert_eq!(human_size(0), "0");
        assert_eq!(human_size(5), "5");
        assert_eq!(human_size(1000), "1000");
        assert_eq!(human_size(1024), "1.0K");
        assert_eq!(human_size(5000), "4.9K");
        assert_eq!(human_size(10240), "10K");
        assert_eq!(human_size(1024 * 1024), "1.0M");
    }

    #[test]
    fn exclude_patterns_match_suffixes_and_components() {
        let one = |p: &str| vec![p.to_string()];
        // Slash-bearing patterns match component-boundary suffixes.
        assert!(is_excluded("a/b/y", &one("b/y")));
        assert!(is_excluded("a/b", &one("a/b")));
        assert!(is_excluded("a/b/y", &one("b/*")));
        assert!(is_excluded("./a/b", &one("a/*")));
        assert!(is_excluded("a/b/c/z", &one("*/z")));
        assert!(is_excluded("./a/b", &one("./a/b")));
        // ...but not a partial component run or a foreign prefix.
        assert!(!is_excluded("a/b/c/z", &one("b/c")));
        assert!(!is_excluded("a/b", &one("b/*")));
        assert!(!is_excluded("a/b/y", &one("zzz/a/x")));
        // Patterns without `/` match any single component.
        assert!(is_excluded("a/b/y", &one("y")));
        assert!(is_excluded("./a", &one("a")));
        assert!(is_excluded("a\\b", &one("b")));
        assert!(!is_excluded("abc", &one("b")));
        // Globs.
        assert!(is_excluded("a/data.txt", &one("*.txt")));
        assert!(is_excluded("a/b/c", &one("?")));
        assert!(!is_excluded("a/data.log", &one("*.txt")));
    }

    #[test]
    fn max_depth_parses_gnu_number_forms() {
        assert_eq!(parse_max_depth("3"), Some(3));
        assert_eq!(parse_max_depth("0x10"), Some(16));
        assert_eq!(parse_max_depth("010"), Some(8));
        assert_eq!(parse_max_depth(" 2"), Some(2));
        assert_eq!(parse_max_depth("+2"), Some(2));
        assert_eq!(parse_max_depth("0"), Some(0));
        assert_eq!(parse_max_depth(""), None);
        assert_eq!(parse_max_depth("-1"), None);
        assert_eq!(parse_max_depth("2x"), None);
        assert_eq!(parse_max_depth("99999999999999999999"), None);
        assert_eq!(parse_max_depth("0x"), None);
    }

    #[test]
    fn volume_prefixes_identify_drives_and_unc_shares() {
        assert_eq!(volume_prefix(r"\\?\C:\Users\T"), Some("c:".into()));
        assert_eq!(volume_prefix(r"C:\x"), Some("c:".into()));
        assert_eq!(volume_prefix(r"c:\x"), Some("c:".into()));
        assert_eq!(
            volume_prefix(r"\\?\UNC\server\share\x"),
            Some(r"unc\server\share".into())
        );
        assert_eq!(
            volume_prefix(r"\\server\share"),
            Some(r"unc\server\share".into())
        );
        assert_eq!(
            volume_prefix(r"\\?\UNC\SRV\Share"),
            Some(r"unc\srv\share".into())
        );
        assert_eq!(volume_prefix("relative"), None);
        assert_eq!(volume_prefix(r"\\?\pipe\x"), None);
    }

    /// Cross-volume pruning with an injected volume predicate: entries on a
    /// foreign volume are neither counted nor printed, and dropping `-x`
    /// counts them again.
    #[test]
    fn one_file_system_prunes_foreign_volumes() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a").join("far")).unwrap();
        fs::write(dir.path().join("a").join("x"), vec![b'x'; 10]).unwrap();
        fs::write(dir.path().join("a").join("far").join("y"), vec![b'y'; 10]).unwrap();

        let volume_of = |path: &Path| -> Option<String> {
            if path.to_string_lossy().contains("far") {
                Some("vol-b".into())
            } else {
                Some("vol-a".into())
            }
        };

        let mut counter = Counter {
            apparent: true,
            human: false,
            bytes: true,
            all: true,
            max_print: u64::MAX,
            one_file_system: true,
            excludes: Vec::new(),
            root_volume: Some("vol-a".into()),
            volume_of: Box::new(volume_of),
            seen: HashSet::new(),
            seen_dirs: HashSet::new(),
            had_error: false,
        };

        let operand = dir.path().join("a");
        let base = operand_name(&operand);
        let mut out = Vec::new();
        let size = counter
            .total(&operand, &base, 0, &mut out)
            .expect("operand is counted");
        assert_eq!(size, 10);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("a/x"));
        assert!(!text.contains("far"));

        counter.one_file_system = false;
        counter.seen.clear();
        counter.seen_dirs.clear();
        let mut out = Vec::new();
        let size = counter
            .total(&operand, &base, 0, &mut out)
            .expect("operand is counted");
        assert_eq!(size, 20);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("far/y"));
    }
}
