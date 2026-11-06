mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashSet;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, GetCompressedFileSizeW, GetFileInformationByHandle,
};

fn main() {
    let args = Args::parse();
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    // GNU prints one line per subdirectory, children before their parent.
    let mut counter = Counter {
        apparent: args.apparent_size || args.bytes,
        human: args.human_readable,
        bytes: args.bytes,
        summarize: args.summarize,
        seen: HashSet::new(),
        seen_dirs: HashSet::new(),
        had_error: false,
    };

    let mut grand_total = 0;
    for path in &paths {
        // GNU omits the line entirely for an argument that repeats an
        // already-seen entry, so only print when a value was returned.
        if let Some(size) = counter.total(path, false) {
            grand_total += size;
            println!("{}\t{}", counter.display(size), path.display());
        }
    }

    if args.total {
        println!("{}\ttotal", counter.display(grand_total));
    }

    if counter.had_error {
        std::process::exit(1);
    }
}

struct Counter {
    /// Count logical sizes instead of allocated disk space.
    apparent: bool,
    human: bool,
    /// Print raw bytes instead of 1K blocks.
    bytes: bool,
    /// -s: suppress the per-subdirectory lines.
    summarize: bool,
    seen: HashSet<(u32, u64, u64)>,
    /// Directories already visited; a repeated argument is skipped entirely.
    seen_dirs: HashSet<PathBuf>,
    had_error: bool,
}

impl Counter {
    /// Returns `None` when the entry was skipped: an already counted entry, or
    /// an unreadable path (already reported on stderr). Nested directories
    /// print their own line unless -s is in effect; the caller prints operands.
    fn total(&mut self, path: &Path, nested: bool) -> Option<u64> {
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
                        size += self.total(&entry.path(), true).unwrap_or(0);
                    }
                }
                Err(e) => {
                    eprintln!("du: {}: {}", path.display(), e);
                    self.had_error = true;
                }
            }
            if nested && !self.summarize {
                println!("{}\t{}", self.display(size), path.display());
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
        Some(if self.apparent {
            logical
        } else {
            allocated_size(path).unwrap_or(logical)
        })
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
    use super::human_size;

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
}
