use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::cli::Args;

/// Represents a single file entry with its metadata.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub metadata: fs::Metadata,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
}

impl FileEntry {
    /// Create a new FileEntry from a path.
    pub fn from_path(path: &Path, follow_links: bool) -> std::io::Result<Self> {
        let metadata = if follow_links {
            fs::metadata(path)?
        } else {
            fs::symlink_metadata(path)?
        };
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let is_symlink = path.is_symlink();
        let symlink_target = if is_symlink {
            fs::read_link(path).ok().map(|t| t.to_string_lossy().to_string())
        } else {
            None
        };
        let is_dir = metadata.is_dir();
        Ok(FileEntry {
            name,
            path: path.to_path_buf(),
            metadata,
            is_dir,
            is_symlink,
            symlink_target,
        })
    }

    /// The file size in bytes.
    pub fn size(&self) -> u64 {
        self.metadata.len()
    }

    /// The allocated size on disk (approximate via Windows file size).
    pub fn block_count(&self) -> u64 {
        // On Windows, the actual allocated size is not directly exposed via std.
        // We approximate: file size / 512 rounded up (standard block size).
        let size = self.size();
        (size + 511) / 512
    }

    /// File index (inode number on Windows via file index).
    pub fn file_index(&self) -> Option<u64> {
        #[cfg(windows)]
        {
            // file_index() is nightly-only; we fall back to None on stable
            let _ = &self.metadata;
        }
        None
    }

    /// Modified time
    pub fn modified(&self) -> Option<SystemTime> {
        self.metadata.modified().ok()
    }

    /// Access time
    pub fn accessed(&self) -> Option<SystemTime> {
        self.metadata.accessed().ok()
    }

    /// Creation time (used as "status change time" equivalent on Windows)
    pub fn created(&self) -> Option<SystemTime> {
        self.metadata.created().ok()
    }

    /// File attributes flags (Windows-specific)
    pub fn file_attributes(&self) -> u32 {
        self.metadata.file_attributes()
    }

    /// Number of hard links (Windows: for directories it's subdirectory count + 1)
    pub fn nlink(&self) -> u64 {
        if self.is_dir {
            // Count subdirectories as a rough hardlink equivalent
            fs::read_dir(&self.path).map(|entries| entries.count() as u64 + 1).unwrap_or(1)
        } else {
            1
        }
    }
}

/// The sort of time to use for --time options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeField {
    Modified,
    Accessed,
    Created,
}

/// Collect all entries for a given path according to CLI args.
pub fn collect_entries(path: &Path, args: &Args) -> std::io::Result<Vec<FileEntry>> {
    if args.directory {
        // List the directory itself, not its contents
        let entry = FileEntry::from_path(path, false)?;
        return Ok(vec![entry]);
    }

    if !path.is_dir() {
        // It's a file — just return it
        let entry = FileEntry::from_path(path, false)?;
        return Ok(vec![entry]);
    }

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path)?;

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        // Filter hidden files (starting with .)
        if !args.show_hidden() && file_name.starts_with('.') {
            continue;
        }

        // Handle --almost-all: show . files but not . and ..
        if args.almost_all && !args.all {
            if file_name == "." || file_name == ".." {
                continue;
            }
        }

        // Handle --ignore-backups: skip files ending with ~
        if args.ignore_backups && file_name.ends_with('~') {
            continue;
        }

        let fe = FileEntry::from_path(&path, false)?;
        entries.push(fe);
    }

    // Apply sorting
    crate::sort::sort_entries(&mut entries, args);

    Ok(entries)
}
